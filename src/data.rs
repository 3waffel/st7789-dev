use sysinfo::{ComponentExt, CpuExt, DiskExt, System, SystemExt};

pub fn get_system_info(sys: &mut System) -> Vec<String> {
    sys.refresh_cpu();
    sys.refresh_memory();
    sys.refresh_components();
    let mut data: Vec<String> = vec![];
    data.append(
        &mut sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| format!("CPU{i}: {:.1}%", cpu.cpu_usage()))
            .collect::<Vec<_>>(),
    );
    data.push(format!(
        "MEM: {:.1}G/{:.1}G",
        sys.used_memory() as f32 / 1024_i32.pow(3) as f32,
        sys.total_memory() as f32 / 1024_i32.pow(3) as f32
    ));
    data.append(
        &mut sys
            .components()
            .iter()
            .map(|c| {
                format!(
                    "{:.3}: {:.1}C/{:.1}C",
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
                    "{:3} {:.1}G/{:.1}G",
                    format!(
                        "{} ({})",
                        disk.mount_point().display(),
                        disk.name().to_str().unwrap_or_default()
                    ),
                    (disk.total_space() - disk.available_space()) as f32 / 1024_i32.pow(3) as f32,
                    disk.total_space() as f32 / 1024_i32.pow(3) as f32
                )
            })
            .collect::<Vec<_>>(),
    );
    data
}
