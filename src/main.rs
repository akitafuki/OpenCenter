mod api;
mod cli;
mod config;
mod discovery;
mod gui;
mod models;
mod tray;

use clap::Parser;
use cli::{Cli, Commands};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Gui) | None => {
            let rt = Arc::new(Runtime::new()?);
            let _tray_handle = tray::spawn_tray(rt);

            if let Err(e) = gui::run_gui() {
                eprintln!("Failed to start GUI: {}", e);
            }
        }
        _ => {
            let rt = Runtime::new()?;
            rt.block_on(async { cli::handle_cli(cli).await })?;
        }
    }

    Ok(())
}
