mod collector;
mod network;
mod tree;
mod tui;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "procmap", about = "Visual process & network topology for your machine")]
struct Cli {
    /// Print a static ASCII process tree to stdout instead of the interactive TUI
    #[arg(long)]
    tree: bool,

    /// Filter to processes owning ports in this range (e.g. 3000-9000)
    #[arg(long, value_name = "START-END")]
    ports: Option<String>,
}

fn parse_port_range(s: &str) -> Result<(u16, u16)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        anyhow::bail!("port range must be in the form START-END, e.g. 3000-9000");
    }
    let start: u16 = parts[0].parse()?;
    let end: u16 = parts[1].parse()?;
    if start > end {
        anyhow::bail!("start port must be <= end port");
    }
    Ok((start, end))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let port_range = cli
        .ports
        .as_deref()
        .map(parse_port_range)
        .transpose()?;

    // Collect data.
    let procs = collector::collect_processes();
    let net = network::collect_network()?;
    let forest = tree::build_tree(&procs, &net, port_range);

    if cli.tree {
        // Static ASCII output.
        let output = tree::render_ascii(&forest);
        print!("{}", output);
    } else {
        // Interactive TUI.
        tui::run_tui(forest)?;
    }

    Ok(())
}
