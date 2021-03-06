use std::sync::Arc;

use clap::{Parser, Subcommand};
use eyre::Result;
use tokio::signal;

use crate::global_state::GlobalState;

mod config;
mod global_state;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// This programs config file location
    #[clap(short, long, default_value_t = String::from("config.toml"))]
    config_file: String,

    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Run the server
    Run,
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    println!("signal received, starting graceful shutdown");
}

async fn run(global_state: Arc<GlobalState>) -> Result<()> {
    println!("Hello, you can gracefully stop this program with Ctrl-C. Reload its config by sending the SIGUSR1 signal.");
    loop {
        // Use global_state like so:
        let state = global_state.load();
        // This is a reference to the GlobalState at the time calling load(). Any subsequent config
        // changes will only be visiible after you dropped the value and called load() again.
        println!("{}", state.config.message);
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let cli = Cli::parse();
    let config_path = cli.config_file.into();
    let global_state = GlobalState::new(config_path).await?;

    tokio::select! {
    _ = shutdown_signal() => {
        println!("Bye!");
    }
    result = match cli.command {
        Commands::Run => run(global_state),
    } => {
            result?;
    }
    }
    Ok(())
}
