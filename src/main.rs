use clap::{Parser, Subcommand};
use std::env;
use surrealvm::commands::*;

fn main() -> anyhow::Result<()> {
    if let Some(name) = env::args().next() {
        if name.ends_with("surreal") {
            eprintln!("surreal version has not been configured, try surrealvm list");
            return Ok(());
        }
    }

    let cli = CLI::parse();

    match cli.command {
        SubCommands::Setup => setup(),
        SubCommands::Clean => clean(),
        SubCommands::List => list(),
        SubCommands::Install { version } => install(version),
        SubCommands::Use { version } => vuse(version),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    /// setup SurrealVM directory with copy of binary and configures PATH
    Setup,
    /// completely uninstally SurrealVM and removes itself from PATH
    Clean,
    /// list installed SurrealDB versions
    #[command(alias = "ls")]
    List,
    /// install specified SurrealDB version
    Install {
        #[arg(value_name = "VERSION", default_value_t = String::from("latest"))]
        version: String,
    },
    /// use specified SurrealDB version
    Use {
        #[arg(value_name = "VERSION", default_value_t = String::from("latest"))]
        version: String,
    },
}
