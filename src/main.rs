mod app;
mod attach;
mod cli;
mod ipc;
mod protocol;
mod registry;
mod runtime;
mod screen;
mod session;
mod winpty;

use std::path::PathBuf;

use anyhow::Result;

#[tokio::main]
async fn main() {
    if let Err(error) = try_main().await {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

async fn try_main() -> Result<()> {
    let cli = cli::Cli::parse_from_env()?;
    match cli.command {
        cli::Command::Serve { session_key, cwd } => {
            runtime::serve(session_key, cwd).await?;
        }
        command => {
            app::run(
                command,
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            )
            .await?;
        }
    }
    Ok(())
}
