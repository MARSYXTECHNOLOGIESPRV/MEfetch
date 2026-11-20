use sysinfo::{
    CpuRefreshKind, Gpus, MemoryRefreshKind, Networks, RefreshKind, System, Disks
};

fn get_system_info() -> System {
    let refresh_kind = RefreshKind::nothing()
        .with_cpu(CpuRefreshKind::everything())
        .with_memory(MemoryRefreshKind::everything());

    match System::new_with_specifics(refresh_kind) {
        Ok(mut system) => {
            system.refresh_cpu_all();
            system.refresh_memory();
            system
        }
        Err(_) => {
            let mut system = System::new().expect("Failed to initialize system information");
            system.refresh_cpu_all();
            system.refresh_memory();
            system
        }
    }
}

fn get_network_summary() -> String {
    let networks = match Networks::new_with_refreshed_list() {
        Ok(networks) => networks,
        Err(_) => return "Unavailable".to_string(),
    };

    let summary: Vec<String> = (&networks)
        .into_iter()
        .map(|(name, data)| {
            format!(
                "{} (D: {} B, U: {} B)",
                name.to_string_lossy(),
                data.received(),
                data.transmitted()
            )
        })
        .collect();

    if summary.is_empty() {
        "None".to_string()
    } else {
        summary.join(" | ")
    }
}

fn print_mefetch(system: &System) {
    let physical_cores = sysinfo::System::physical_core_count().unwrap_or(0);
    let os_name = System::name().unwrap_or_else(|_| "Unknown OS".to_string());
    let os_version = System::os_version().unwrap_or_else(|_| "Unknown".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|_| "Unknown".to_string());
    let host = System::host_name().unwrap_or_else(|_| "Unknown".to_string());
    let network = get_network_summary();
    let total_ram_mb = system.total_memory() / 1024 / 1024;
    let disks = match Disks::new_with_refreshed_list() {
        Ok(disks) => disks
            .iter()
            .find(|disk| disk.mount_point() == std::path::Path::new("/"))
            .map(|disk| {
                format!(
                    "PATH: {} | FS: {} | TYPE: {:?} | MAX: {:.1} GiB",
                    disk.name().to_string_lossy(),
                    disk.file_system().to_string_lossy(),
                    disk.kind(),
                    disk.total_space() as f64 / 1024.0_f64.powi(3)
                )
            })
            .unwrap_or_else(|| "None".to_string()),
        Err(_) => "Unavailable".to_string(),
    };
    let displays = display_info::DisplayInfo::all().unwrap();



    let cpu = system
        .cpus()
        .first()
        .map(|cpu| format!("{} {} {} GHz", cpu.name(), cpu.brand(), cpu.frequency() as f64 / 1000.0))
        .unwrap_or_else(|| "Unknown".to_string());

    let (gpu, gpu_pci) = match Gpus::new_with_refreshed_list() {
        Ok(gpus) => match gpus.first() {
            Some(gpu) => (
                format!(
                    "{} ({})",
                    gpu.model().unwrap_or("Unknown model"),
                    gpu.vendor().unwrap_or("Unknown vendor")
                ),
                gpu.pci().to_string(),
            ),
            None => ("Not detected".to_string(), "Unavailable".to_string()),
        },
        Err(_) => ("Unavailable".to_string(), "Unavailable".to_string()),
    };

    println!(
        r#"[OK] MEfetch
                     .::.    | OS Version: {} {}
                  .:'  .:    | Kernel Version: {}
        ,MMM8&&&.:'   .:'    | Host Name: {}
       MMMMM88&&&&  .:'      | Network: {}
      MMMMM88&&&&&&:'        | CPU: {} ({} Physical Cores)
      MMMMM88&&&&&&          | GPU: {}
    .:MMMMM88&&&&&&          | PCI: {}
  .:'  MMMMM88&&&&           | RAM: {} MiB
.:'   .:'MMM8&&&'            | DISKS: {}
:'  .:'                      | DISPLAYS: {}
'::'                         |"#,
        os_name, os_version, kernel, host, network, cpu, physical_cores, gpu, gpu_pci, total_ram_mb, disks, displays
            .iter()
            .map(|display| {
                format!(
                    "Display: {} | Resolution: {}x{} | Refresh Rate: {} Hz",
                    display.name,
                    display.width,
                    display.height,
                    display.frequency
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    );
}

fn main() {
    let system = get_system_info();
    print_mefetch(&system);
}
