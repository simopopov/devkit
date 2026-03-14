use sysinfo::{System, Users};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub user: String,
    pub cpu_percent: f32,
    pub mem_percent: f32,
}

/// Collect all running processes using sysinfo.
pub fn collect_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    // Refresh twice to get meaningful CPU readings.
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_all();

    let users = Users::new_with_refreshed_list();
    let user_map: HashMap<_, _> = users
        .iter()
        .map(|u| (u.id().clone(), u.name().to_string()))
        .collect();

    let total_mem = sys.total_memory() as f64;

    sys.processes()
        .values()
        .map(|p| {
            let uid = p.user_id();
            let user = uid
                .and_then(|id| user_map.get(id))
                .cloned()
                .unwrap_or_else(|| String::from("-"));
            let mem_pct = if total_mem > 0.0 {
                (p.memory() as f64 / total_mem * 100.0) as f32
            } else {
                0.0
            };
            ProcessInfo {
                pid: p.pid().as_u32(),
                ppid: p.parent().map(|pp| pp.as_u32()).unwrap_or(0),
                name: p.name().to_string_lossy().to_string(),
                user,
                cpu_percent: p.cpu_usage(),
                mem_percent: mem_pct,
            }
        })
        .collect()
}
