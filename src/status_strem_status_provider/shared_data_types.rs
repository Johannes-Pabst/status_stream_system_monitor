use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RequestWrapper {
    pub request: Request,
    pub rid: RID,
    pub auth: ClientAuth,
}
#[derive(Serialize, Deserialize, Clone)]
pub enum Request {
    InformationCollector(InformationPacket),
    Filler,
}
pub type RID = u32;
pub type GID = u32;
#[derive(Serialize, Deserialize, Clone)]
pub struct ClientAuth {
    pub api_key: Option<String>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct InformationPacket {
    pub status: TestResult,
    pub data_points: Vec<Vec<DataPoint>>,
    pub graph_summaries: Vec<GraphSummary>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct DataPoint {
    pub timestamp: i64,
    pub value: f64,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct GraphSummary {
    pub name: String,
    pub description: String,
    pub max:Option<f64>,
    pub min:Option<f64>,
    pub unit:String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct TestResult {
    pub status: TestStatus,
    pub message: String,
    pub details: String,
}
pub trait ErrorVerbose: Error {
    fn verbose(&self) -> String;
}
impl ErrorVerbose for reqwest::Error {
    fn verbose(&self) -> String {
        if self.is_body() {
            return format!("Error with request body: {}", self);
        }
        if self.is_builder() {
            return format!("Error with request builder: {}", self);
        }
        if self.is_connect() {
            return format!("Error connecting to server: {}", self);
        }
        if self.is_decode() {
            return format!("Error decoding response: {}", self);
        }
        if self.is_redirect() {
            return format!("Error with redirect: {}", self);
        }
        if self.is_request() {
            return format!("Error with request: {}", self);
        }
        if self.is_status() {
            return format!("Error with status: {}", self);
        }
        if self.is_timeout() {
            return format!("Timeout: {}", self);
        }
        format!("{self}")
    }
}
impl TestResult {
    pub fn stringify(&self) -> String {
        format!(
            "status: {}\nmessage: {}\ndetails:\n{}",
            self.status.stringify(),
            self.message,
            self.details
        )
    }
    pub fn internal_error(e: impl ErrorVerbose) -> Self {
        TestResult {
            status: TestStatus::InternalError,
            message: e.verbose(),
            details: "".to_string(),
        }
    }
    pub fn ok() -> Self {
        TestResult {
            status: TestStatus::Ok,
            message: "".to_string(),
            details: "".to_string(),
        }
    }
    pub fn warn<T>(msg: T) -> Self
    where
        String: From<T>,
    {
        TestResult {
            status: TestStatus::Warning,
            message: String::from(msg),
            details: "".to_string(),
        }
    }
    pub fn err<T>(msg: T) -> Self
    where
        String: From<T>,
    {
        TestResult {
            status: TestStatus::Error,
            message: String::from(msg),
            details: "".to_string(),
        }
    }
    pub fn fatal<T>(msg: T) -> Self
    where
        String: From<T>,
    {
        TestResult {
            status: TestStatus::Fatal,
            message: String::from(msg),
            details: "".to_string(),
        }
    }
    pub fn msg<T>(mut self, msg: T) -> Self
    where
        String: From<T>,
    {
        self.message = String::from(msg);
        self
    }
    pub fn dets<T>(mut self, dets: T) -> Self
    where
        String: From<T>,
    {
        self.message = String::from(dets);
        self
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TestStatus {
    Ok,
    Warning,
    Error,
    Fatal,

    InternalError,
}
impl TestStatus {
    pub fn stringify(&self) -> String {
        match self {
            TestStatus::Ok => "OK",
            TestStatus::Warning => "WARNING",
            TestStatus::Error => "ERROR",
            TestStatus::Fatal => "FATAL",
            TestStatus::InternalError => "INTERNAL ERROR",
        }
        .to_string()
    }
}
