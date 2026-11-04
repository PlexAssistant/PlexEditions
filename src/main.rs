mod api;
mod config;
mod error;
mod model;
mod modules;
mod run;
mod ws;

use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use tracing::info;

use crate::config::Config;
use crate::error::Result;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Command {
    /// Write editions for movies that do not have one, then exit.
    All,
    /// Listen for new movies and write an edition for each.
    New,
    /// Clear editions in movie libraries, then exit.
    Reset,
}

#[derive(Parser)]
#[command(
    name = "editions",
    about = "Write movie edition titles on a Plex server"
)]
struct Cli {
    #[arg(value_enum)]
    command: Command,
    /// Print the edition that would be written. Do not change the server.
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .without_time()
        .init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::from_env()?;
    config.dry_run = cli.dry_run;
    let client = api::Client::new(&config.plex_url, &config.plex_token)?;
    let name = client.friendly_name().await?;
    match config.language {
        config::Language::Zh => info!("已成功连接到服务器：{name}"),
        config::Language::En => info!("Successfully connected to server: {name}"),
    }
    match cli.command {
        Command::All => {
            let stats = run::run(&config, &client, run::Mode::All).await?;
            info!(
                written = stats.written,
                skipped = stats.skipped,
                failed = stats.failed,
                "done"
            );
        }
        Command::Reset => {
            let stats = run::run(&config, &client, run::Mode::Reset).await?;
            info!(
                cleared = stats.written,
                skipped = stats.skipped,
                failed = stats.failed,
                "done"
            );
        }
        Command::New => ws::listen(&config, &client).await?,
    }
    Ok(())
}
