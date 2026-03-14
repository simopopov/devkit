use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct NetConn {
    pub proto: String,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: String, // LISTEN, ESTABLISHED, etc.
}

/// Parse `lsof -i -P -n` output and return a map from PID to its network connections.
pub fn collect_network() -> Result<HashMap<u32, Vec<NetConn>>> {
    let output = Command::new("lsof")
        .args(["-i", "-P", "-n"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(HashMap::new()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut map: HashMap<u32, Vec<NetConn>> = HashMap::new();

    for line in stdout.lines().skip(1) {
        // lsof columns are variable-width; split on whitespace.
        // Typical: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        let pid: u32 = match parts[1].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let proto = parts[8].to_string(); // TCP or UDP
        let name_field = parts[9]; // e.g. 127.0.0.1:8080->10.0.0.1:443 or *:3000

        let state = if parts.len() > 10 {
            parts[10].trim_start_matches('(').trim_end_matches(')').to_string()
        } else {
            String::new()
        };

        let (local, remote) = if let Some(idx) = name_field.find("->") {
            (&name_field[..idx], &name_field[idx + 2..])
        } else {
            (name_field, "")
        };

        let (local_addr, local_port) = parse_host_port(local);
        let (remote_addr, remote_port) = parse_host_port(remote);

        map.entry(pid).or_default().push(NetConn {
            proto,
            local_addr,
            local_port,
            remote_addr,
            remote_port,
            state,
        });
    }

    Ok(map)
}

fn parse_host_port(s: &str) -> (String, u16) {
    if s.is_empty() {
        return (String::new(), 0);
    }
    // Could be [::1]:port, *:port, 127.0.0.1:port, or just *:*
    if let Some(idx) = s.rfind(':') {
        let addr = &s[..idx];
        let port = s[idx + 1..].parse::<u16>().unwrap_or(0);
        (addr.to_string(), port)
    } else {
        (s.to_string(), 0)
    }
}
