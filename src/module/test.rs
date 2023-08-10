use std::time::{SystemTime, Duration, Instant};
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};

// use crate::model::{AgentStore, AgentManage, AgentHistory};

#[allow(dead_code)]
fn get_fn_name<F>(_: F) -> &'static str 
where
    F: Fn(),
{
    std::any::type_name::<F>()
}

pub fn time_function<F: FnOnce() -> T, T>(func: F, name: &'static str) -> T {
    let start_time = SystemTime::now();
    let result = func();
    let end_time = SystemTime::now();
    let duration = end_time.duration_since(start_time).unwrap();
    println!("Time Fn #{}# : {:?} miliseconds", name, duration.as_millis());
    result
}

pub fn benchmark_env_usage(intervel: Duration) -> (f32, f64, f64, f64) {
    let mut system = System::new_all();

    system.refresh_all();

    let start_cpu_usage = system.global_cpu_info().cpu_usage();
    let start_ram_usage = system.used_memory();
    
    let start_time = Instant::now();

    while Instant::now() - start_time < intervel {
        std::thread::sleep(Duration::from_millis(100));
        system.refresh_all();
    }

    let end_cpu_usage = system.global_cpu_info().cpu_usage();
    let elapsed_time = start_time.elapsed().as_secs_f32();

    let end_ram_usage = system.used_memory();

    let cpu_usage = ((end_cpu_usage - start_cpu_usage) / elapsed_time) * -1.0 / 100.0;
    let ram_usage = (end_ram_usage - start_ram_usage) as f64 * 1e-6;
    let find_disk_usage: Vec<(bool, u64, u64)> = system.processes()
        .iter()
        .map(|(_, process)| 
            match process.name() == "agent_client" {
                true => {
                    let dsk = process.disk_usage();
                    (true, dsk.total_read_bytes, dsk.total_written_bytes)
                },
                false => (false, 0, 0)
            })
        .collect();
    let (read_usage, write_usage) = find_disk_usage.iter().find(|&&(b, _, _)| b).map(|(_, r, w)| (*r as f64 * 1e-6, *w as f64 * 1e-6)).unwrap();

    (cpu_usage, ram_usage, read_usage, write_usage)
}
