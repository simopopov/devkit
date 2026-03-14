use anyhow::{bail, Context, Result};
use colored::Colorize;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crate::scanner;

/// Kill processes on a given port with SIGTERM, then SIGKILL after timeout.
pub fn kill_port(port: u16, skip_confirm: bool) -> Result<()> {
    let entries = scanner::scan_port(port)?;

    if entries.is_empty() {
        println!("{}", format!("No process found on port {port}").yellow());
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} process(es) on port {port}:", entries.len()).bold()
    );
    for e in &entries {
        println!(
            "  PID {} — {} (user: {})",
            e.pid.to_string().cyan(),
            e.process.green(),
            e.user
        );
    }

    if !skip_confirm {
        print!("{}", "Kill these processes? [y/N] ".red().bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    for e in &entries {
        kill_pid(e.pid)?;
    }

    Ok(())
}

/// Free an entire port range.
pub fn free_range(start: u16, end: u16, skip_confirm: bool) -> Result<()> {
    let entries = scanner::scan_port_range(start, end)?;

    if entries.is_empty() {
        println!(
            "{}",
            format!("No processes found on ports {start}-{end}").yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "Found {} process(es) on ports {start}-{end}:",
            entries.len()
        )
        .bold()
    );
    for e in &entries {
        println!(
            "  :{} — PID {} — {} (user: {})",
            e.port.to_string().yellow(),
            e.pid.to_string().cyan(),
            e.process.green(),
            e.user
        );
    }

    if !skip_confirm {
        print!("{}", "Kill all these processes? [y/N] ".red().bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Deduplicate PIDs (same process may hold multiple ports)
    let mut pids: Vec<u32> = entries.iter().map(|e| e.pid).collect();
    pids.sort();
    pids.dedup();

    for pid in pids {
        kill_pid(pid)?;
    }

    Ok(())
}

/// Send SIGTERM, wait 3 seconds, then SIGKILL if still alive.
pub fn kill_pid(pid: u32) -> Result<()> {
    let nix_pid = Pid::from_raw(pid as i32);

    println!("Sending SIGTERM to PID {}...", pid.to_string().cyan());
    match kill(nix_pid, Signal::SIGTERM) {
        Ok(()) => {}
        Err(nix::errno::Errno::ESRCH) => {
            println!("  PID {pid} already gone.");
            return Ok(());
        }
        Err(nix::errno::Errno::EPERM) => {
            bail!("Permission denied killing PID {pid}. Try running with sudo.");
        }
        Err(e) => {
            bail!("Failed to send SIGTERM to PID {pid}: {e}");
        }
    }

    // Wait up to 3 seconds for process to exit
    for _ in 0..6 {
        thread::sleep(Duration::from_millis(500));
        if kill(nix_pid, None).is_err() {
            println!("  PID {pid} terminated {}.", "successfully".green());
            return Ok(());
        }
    }

    // Still alive — escalate to SIGKILL
    println!(
        "  PID {pid} did not exit, sending {}...",
        "SIGKILL".red().bold()
    );
    kill(nix_pid, Signal::SIGKILL).context(format!("Failed to SIGKILL PID {pid}"))?;

    thread::sleep(Duration::from_millis(500));
    if kill(nix_pid, None).is_err() {
        println!("  PID {pid} killed {}.", "successfully".green());
    } else {
        println!(
            "  {}",
            format!("Warning: PID {pid} may still be running").yellow()
        );
    }

    Ok(())
}
