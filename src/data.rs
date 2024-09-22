use sysinfo::{ComponentExt, CpuExt, DiskExt, System, SystemExt};

pub fn get_header_info() -> String {
    let system = System::new_all();
    let uptime = system.uptime();
    let version = system.long_os_version().unwrap();
    format!(
        "{}:{:02}:{:02}\t{}",
        uptime / 3600,
        (uptime % 3600) / 60,
        (uptime % 60),
        version
    )
}

pub fn get_system_info() -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    sys.refresh_memory();
    sys.refresh_components();
    let mut data: Vec<String> = vec![];
    data.append(
        &mut sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| format!("CPU{i}:\t{:.1}%", cpu.cpu_usage()))
            .collect::<Vec<_>>(),
    );
    data.push(format!(
        "MEM:\t{:.1}G/{:.1}G",
        sys.used_memory() as f32 / 1024_i32.pow(3) as f32,
        sys.total_memory() as f32 / 1024_i32.pow(3) as f32
    ));
    data.append(
        &mut sys
            .components()
            .iter()
            .map(|c| {
                format!(
                    "{:.3}:\t{:.1}°C/{:.1}°C",
                    c.label(),
                    c.temperature(),
                    c.critical().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>(),
    );
    data.append(
        &mut sys
            .disks()
            .iter()
            .map(|disk| {
                format!(
                    "{:3}\t{:.1}G/{:.1}G",
                    // format!(
                    //     "{}\t({})",
                    //     disk.mount_point().display(),
                    //     disk.name().to_str().unwrap_or_default()
                    // ),
                    disk.mount_point().display(),
                    (disk.total_space() - disk.available_space()) as f32 / 1024_i32.pow(3) as f32,
                    disk.total_space() as f32 / 1024_i32.pow(3) as f32
                )
            })
            .collect::<Vec<_>>(),
    );
    data
}
