use std::{mem, sync::Arc, time::Duration};

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, task::JoinHandle};

use super::{
    auth::get_access_token,
    shared_data_types::{
        DataPoint, GraphSummary, InformationPacket, Request, RequestWrapper, TestResult, RID,
    },
};
#[derive(Clone)]
pub struct CommunicationsManager {
    pub config: Arc<RwLock<CommunicationsConfig>>,
    buffer: Arc<RwLock<InformationPacket>>,
    buffered_update_calls: Arc<RwLock<usize>>,
    last_pos_send_time: Arc<RwLock<i64>>,
    cur_send_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    jwt: Arc<RwLock<(String, i64)>>,
}
impl CommunicationsManager {
    pub fn new(config: CommunicationsConfig, gs: Vec<GraphSummary>) -> Self {
        CommunicationsManager {
            config: Arc::new(RwLock::new(config)),
            buffer: Arc::new(RwLock::new(InformationPacket {
                status: TestResult::ok(),
                data_points: Vec::new(),
                graph_summaries: gs,
            })),
            last_pos_send_time: Arc::new(RwLock::new(i64::MAX)),
            buffered_update_calls: Arc::new(RwLock::new(0)),
            cur_send_task: Arc::new(RwLock::new(None)),
            jwt: Arc::new(RwLock::new((String::new(), 0))),
        }
    }
    pub async fn update_status(&self, status: TestResult, max_delay: i64) {
        if self.buffer.read().await.status != status {
            self.buffer.write().await.status = status;
            self.update_await(max_delay).await;
        }
    }
    pub async fn update_data_points(&self, mut data_points: Vec<Vec<DataPoint>>, max_delay: i64) {
        if data_points.len() > 0 {
            {
                let mut buf = self.buffer.write().await;
                for (i, g) in data_points.iter_mut().enumerate() {
                    if buf.data_points.len() <= i {
                        buf.data_points.push(g.clone());
                    } else {
                        buf.data_points[i].append(g);
                    }
                }
            }
            let mut buc = self.buffered_update_calls.write().await;
            *buc += 1;
            if *buc >= self.config.read().await.max_buffered_update_calls {
                self.send().await;
                *buc = 0;
            } else {
                self.update_await(max_delay).await;
            }
        }
    }
    async fn update_await(&self, max_delay: i64) {
        if max_delay == 0 {
            self.send().await;
        } else if max_delay == i64::MAX {
            return;
        } else {
            let lpst = chrono::Utc::now().timestamp() + max_delay;
            let last_pos_send_time = *self.last_pos_send_time.read().await;
            if lpst < last_pos_send_time {
                *self.last_pos_send_time.write().await = lpst;
                let mut st = self.cur_send_task.write().await;
                if let Some(sts) = st.as_ref() {
                    sts.abort();
                }
                let self2 = self.clone();
                *st = Some(tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(
                        (lpst - chrono::Utc::now().timestamp()) as u64,
                    ))
                    .await;
                    self2.send_raw().await;
                }));
            }
        }
    }
    async fn send(&self) {
        {
            let st = self.cur_send_task.write().await;
            if let Some(sts) = st.as_ref() {
                sts.abort();
            }
        }
        self.send_raw().await;
    }
    async fn send_raw(&self) {
        let self2 = self.clone();
        tokio::spawn(async move {
            let mut buf;
            {
                let mut buffer = self2.buffer.write().await;
                buf = InformationPacket {
                    status: buffer.status.clone(),
                    data_points: Vec::new(),
                    graph_summaries: buffer.graph_summaries.clone(),
                };
                let mut task_guard: tokio::sync::RwLockWriteGuard<'_, Option<JoinHandle<()>>> =
                    self2.cur_send_task.write().await;
                *task_guard = None;
                mem::swap(&mut buf, &mut buffer);
            }
            if self2.send_raw_raw(&buf).await.is_err() {
                let mut buffer = self2.buffer.write().await;
                for i in 0..(buf.data_points.len()) {
                    if i < buffer.data_points.len() {
                        buffer.data_points[i].splice(0..0, buf.data_points[i].clone());
                    } else {
                        buffer.data_points.push(buf.data_points[i].clone());
                    }
                }
            }
        });
    }
    async fn send_raw_raw(&self, buffer: &InformationPacket) -> Result<(), ()> {
        let config = self.config.read().await;
        let data = serde_json::to_string(&RequestWrapper {
            request: Request::InformationCollector(buffer),
            rid: config.rid.clone(),
        })
        .map_err(|e| {
            eprintln!("{e}");
            ()
        })?;
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();

        let req_builder = client
            .post(format!("{}/update_test", config.api_endpoint))
            .body(data);
        let now = chrono::Utc::now().timestamp();
        let read = self.jwt.read().await;
        let jwt = if read.1 <= now + 10 {
            drop(read);
            let t = get_access_token(config.clone()).await.map_err(|e| {
                eprintln!("{e}");
                ()
            })?;
            *self.jwt.write().await = (t.0.clone(), t.1 + now);
            t.0
        } else {
            read.0.clone()
        };
        headers.append(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", jwt).as_str()).map_err(|e| {
                eprintln!("{e}");
                ()
            })?,
        );
        headers.append(
            "X_Realm",
            HeaderValue::from_str(&config.kc_realm).map_err(|e| {
                eprintln!("{e}");
                ()
            })?,
        );
        let response = req_builder
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                eprintln!("{e}");
                ()
            })?;
            let s=response.status();
            let o = response
                .text()
                .await
                .map_err(|e| {
                    eprintln!("{e}");
                    ()
                })?;
            if !s.is_success() {
                if o.len() > 0 {
                    eprintln!("response: {}", o);
                }
                eprintln!("status code: {} {}", s.as_u16(),s.as_str());
                return Err(());
            };
        Ok(())
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct CommunicationsConfig {
    pub api_endpoint: String,
    pub kc_realm: String,
    pub kc_client_secret: String,
    pub kc_client_id: String,
    pub kc_url: String,
    pub rid: RID,
    pub max_buffered_update_calls: usize,
}
