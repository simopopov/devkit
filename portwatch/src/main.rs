mod killer;
mod scanner;
mod tui;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "pw", about = "Port monitoring & conflict resolution for macOS")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Query a specific port, e.g. `:3000`
    #[arg(value_name = "PORT")]
    port: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Kill the process holding a port (e.g. `pw kill :3000`)
    Kill {
        /// Port to kill, e.g. `:3000`
        port: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Free an entire port range (e.g. `pw free 3000-3010`)
    Free {
        /// Port range, e.g. `3000-3010`
        range: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

fn parse_port(s: &str) -> Result<u16> {
    let s = s.strip_prefix(':').unwrap_or(s);
    let port: u16 = s
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid port: {s}"))?;
    Ok(port)
}

fn parse_range(s: &str) -> Result<(u16, u16)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        bail!("Invalid range format. Use START-END, e.g. 3000-3010");
    }
    let start: u16 = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid start port: {}", parts[0]))?;
    let end: u16 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid end port: {}", parts[1]))?;
    if start > end {
        bail!("Start port must be <= end port");
    }
    Ok((start, end))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Kill { port, yes }) => {
            let port = parse_port(&port)?;
            killer::kill_port(port, yes)?;
        }
        Some(Commands::Free { range, yes }) => {
            let (start, end) = parse_range(&range)?;
            killer::free_range(start, end, yes)?;
        }
        None => {
            if let Some(port_str) = cli.port {
                // Query a specific port
                let port = parse_port(&port_str)?;
                let entries = scanner::scan_port(port)?;
                if entries.is_empty() {
                    println!("{}", format!("Port {port} is free.").green());
                } else {
                    println!(
                        "{}",
                        format!("Port {port} is in use:").yellow().bold()
                    );
                    print_entries(&entries);
                }
            } else {
                // No args — launch TUI dashboard
                tui::run_tui()?;
            }
        }
    }

    Ok(())
}

fn print_entries(entries: &[scanner::PortEntry]) {
    // Print header
    println!(
        "  {:<7} {:<8} {:<18} {:<12} {:<7} {:<9} {}",
        "PORT".yellow().bold(),
        "PID".yellow().bold(),
        "PROCESS".yellow().bold(),
        "USER".yellow().bold(),
        "CPU%".yellow().bold(),
        "MEM(MB)".yellow().bold(),
        "PROTO".yellow().bold(),
    );

    for e in entries {
        println!(
            "  {:<7} {:<8} {:<18} {:<12} {:<7.1} {:<9.1} {}",
            e.port.to_string().green(),
            e.pid.to_string().cyan(),
            e.process,
            e.user,
            e.cpu,
            e.mem,
            e.proto.magenta(),
        );
    }
}
