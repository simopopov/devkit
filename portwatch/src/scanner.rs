use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PortEntry {
    pub port: u16,
    pub pid: u32,
    pub process: String,
    pub user: String,
    pub proto: String,
    pub cpu: f32,
    pub mem: f64,
}

/// Parse lsof -i -P -n output to extract listening port entries.
pub fn scan_ports() -> Result<Vec<PortEntry>> {
    let output = Command::new("lsof")
        .args(["-i", "-P", "-n"])
        .output()
        .context("Failed to run lsof")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines().skip(1) {
        if let Some(entry) = parse_lsof_line(line) {
            entries.push(entry);
        }
    }

    // Enrich with CPU/MEM from sysinfo
    enrich_entries(&mut entries);

    // Sort by port
    entries.sort_by_key(|e| e.port);
    // Deduplicate by (port, pid)
    entries.dedup_by(|a, b| a.port == b.port && a.pid == b.pid);

    Ok(entries)
}

/// Scan for a specific port.
pub fn scan_port(port: u16) -> Result<Vec<PortEntry>> {
    let entries = scan_ports()?;
    Ok(entries.into_iter().filter(|e| e.port == port).collect())
}

/// Scan for a range of ports.
pub fn scan_port_range(start: u16, end: u16) -> Result<Vec<PortEntry>> {
    let entries = scan_ports()?;
    Ok(entries
        .into_iter()
        .filter(|e| e.port >= start && e.port <= end)
        .collect())
}

fn parse_lsof_line(line: &str) -> Option<PortEntry> {
    // lsof output columns (space-separated, variable width):
    // COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 10 {
        return None;
    }

    let process = parts[0].to_string();
    let pid: u32 = parts[1].parse().ok()?;
    let user = parts[2].to_string();
    let proto_raw = parts[7]; // NODE column = TCP/UDP

    // NAME column is the last part, e.g. "*:3000 (LISTEN)" or "127.0.0.1:8080->..."
    // We join remaining parts from index 8 onward
    let name = parts[8..].join(" ");

    // Only include LISTEN entries for TCP, or UDP entries
    let is_listen = name.contains("(LISTEN)");
    let is_udp = proto_raw.eq_ignore_ascii_case("UDP");

    if !is_listen && !is_udp {
        return None;
    }

    // Extract port from the name field: "*:3000" or "127.0.0.1:3000" or "[::1]:3000"
    let addr_part = name.split_whitespace().next()?;
    let port_str = addr_part.rsplit(':').next()?;
    // Handle arrow notation for connected sockets
    let port_str = port_str.split("->").next()?;
    let port: u16 = port_str.parse().ok()?;

    let proto = if is_udp {
        "UDP".to_string()
    } else {
        "TCP".to_string()
    };

    Some(PortEntry {
        port,
        pid,
        process,
        user,
        proto,
        cpu: 0.0,
        mem: 0.0,
    })
}

fn enrich_entries(entries: &mut [PortEntry]) {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    for entry in entries.iter_mut() {
        let pid = sysinfo::Pid::from(entry.pid as usize);
        if let Some(proc_info) = sys.process(pid) {
            entry.cpu = proc_info.cpu_usage();
            entry.mem = proc_info.memory() as f64 / (1024.0 * 1024.0); // MB
        }
    }
}
