pub mod status_strem_status_provider;
use serde::{Deserialize, Serialize};
use status_strem_status_provider::{
    communications::{CommunicationsConfig, CommunicationsManager},
    config::Config,
    shared_data_types::GraphSummary,
    utils::ask_yn,
};
use sysinfo::{Components, Disks, Networks, System};
#[tokio::main]
async fn main() {
    println!("Hello, World!");
    let configpath = "config.toml";
    let server_config = MonitorConfig::load(configpath).unwrap_or_else(|_| {
        if ask_yn(
            format!("couldn't find file {configpath}. create file?"),
            true,
        ) {
            MonitorConfig {
                com_config: CommunicationsConfig {
                    api_endpoint: "127.0.0.1:7070/update_test".to_string(),
                    api_key: None,
                    max_buffered_update_calls: 100,
                    rid: 20,
                },
                ram: true,
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
    let gs: Vec<GraphSummary> = Vec::new();
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    // Display system information:
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());

    // Number of CPUs:
    println!("NB CPUs: {}", sys.cpus().len());
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Display processes ID, name na disk usage:
    for (pid, process) in sys.processes() {
        println!("[{pid}] {:?} {:?}", process.name(), process.cpu_usage());
    }

    // We display all disks' information:
    println!("=> disks:");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        println!("{disk:?}");
    }

    // Network interfaces name, total data received and total data transmitted:
    let networks = Networks::new_with_refreshed_list();
    println!("=> networks:");
    for (interface_name, data) in &networks {
        println!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.total_received(),
            data.total_transmitted(),
        );
        // If you want the amount of data received/transmitted since last call
        // to `Networks::refresh`, use `received`/`transmitted`.
    }

    // Components temperature:
    let components = Components::new_with_refreshed_list();
    println!("=> components:");
    for component in &components {
        println!("{component:?}");
    }
    let com_manager = CommunicationsManager::new(server_config.com_config.clone(), gs);
}
#[derive(Serialize, Deserialize)]
struct MonitorConfig {
    com_config: CommunicationsConfig,
    ram: bool,
}
impl Config for MonitorConfig {}
