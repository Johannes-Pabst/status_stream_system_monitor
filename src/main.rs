pub mod status_strem_status_provider;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use status_strem_status_provider::{
    communications::{CommunicationsConfig, CommunicationsManager}, config::Config, shared_data_types::{DataPoint, GraphSummary}, utils::ask_yn
};
use sysinfo::System;
#[tokio::main]
async fn main() {
    let configpath = "config.toml";
    let server_config = MonitorConfig::load(configpath).unwrap_or_else(|_| {
        if ask_yn(
            format!("couldn't find file {configpath}. create file?"),
            true,
        ) {
            MonitorConfig {
                com_config: CommunicationsConfig {
                    api_endpoint: "http://127.0.0.1:7070".to_string(),
                    max_buffered_update_calls: 2,
                    rid: 1,
                    kc_client_id:"self-system-monitor".to_string(),
                    kc_client_secret:"8g3J21qfCAOTYjI2a2vDoG5WLkdNjkG2".to_string(),
                    kc_realm:"self".to_string(),
                    kc_url:"http://172.23.38.25:8080".to_string(),
                },
                ram: true,
                swap: true,
                network: true,
                cores: true,
                cpu: true,
                task_cpu: vec!["chrome.exe".to_string()],
                measurement_delay_secs:10
            }
            .save(configpath)
            .unwrap();
            println!("file {configpath} created!");
            MonitorConfig::load(configpath).unwrap()
        } else {
            println!("aborting...");
            std::process::exit(1);
        }
    });
    let mut sys = System::new_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();
    let (gs, data)=check_system(&server_config,&mut sys);
    let com_manager = CommunicationsManager::new(server_config.com_config.clone(), gs);
    com_manager.update_data_points(data, i64::MAX).await;
    loop{
        tokio::time::sleep(Duration::from_secs(server_config.measurement_delay_secs)).await;
        let (_, data)=check_system(&server_config,&mut sys);
        com_manager.update_data_points(data, i64::MAX).await;
    }
}
fn check_system(server_config: &MonitorConfig,sys:&mut System)->(Vec<GraphSummary>, Vec<Vec<DataPoint>>){
    let mut gs: Vec<GraphSummary> = Vec::new();
    let mut points:Vec<f64>=Vec::new();
    sys.refresh_all();
    if server_config.ram{
        gs.push(GraphSummary{
            name:"RAM".to_string(),
            description:"".to_string(),
            max:Some(sys.total_memory() as f64),
            min:Some(0.0),
            unit:"bytes".to_string(),
        });
        points.push(sys.used_memory() as f64);
    }
    if server_config.swap{
        gs.push(GraphSummary{
            name:"Swap".to_string(),
            description:"".to_string(),
            max:Some(sys.total_swap() as f64),
            min:Some(0.0),
            unit:"bytes".to_string(),
        });
        points.push(sys.used_swap() as f64);
    }
    if server_config.cpu{
        gs.push(GraphSummary{
            name:"CPU".to_string(),
            description:"".to_string(),
            max:Some(100.0),
            min:Some(0.0),
            unit:"%".to_string(),
        });
        points.push(sys.global_cpu_usage() as f64);
    }
    if server_config.cores{
        for cpu in sys.cpus(){
            gs.push(GraphSummary{
            name:cpu.name().to_string(),
            description:"".to_string(),
            max:Some(100.0),
            min:Some(0.0),
            unit:"%".to_string(),
        });
        points.push(cpu.cpu_usage() as f64);
        }
    }
    // let networks = Networks::new_with_refreshed_list();
    
    // for (interface_name, data) in &networks {
    // }
    // let components = Components::new_with_refreshed_list();
    // for component in &components {
    // }
    let time=chrono::Utc::now().timestamp_millis();
    (gs,points.iter().map(|f| vec![DataPoint{timestamp:time,value:*f}]).collect::<Vec<Vec<DataPoint>>>())
}
#[derive(Serialize, Deserialize)]
struct MonitorConfig {
    com_config: CommunicationsConfig,
    ram: bool,
    swap: bool,
    cpu:bool,
    cores:bool,
    network:bool,
    task_cpu:Vec<String>,
    measurement_delay_secs:u64,
}
impl Config for MonitorConfig {}
