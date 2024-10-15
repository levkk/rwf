use clap::{Args, Parser, Subcommand};
use rwf::logging::Logger;

use std::path::Path;

mod logging;
mod migrate;
mod setup;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Migrate(MigrateSubcommand),

    /// Setup the project for Rwf
    Setup,
}

#[derive(Args, Debug)]
struct MigrateSubcommand {
    #[command(subcommand)]
    command: Migrate,
}

/// Manage migrations.
#[derive(Subcommand, Debug)]
enum Migrate {
    /// Run migrations.
    Run {
        #[arg(long, help = "Run migrations up to this version")]
        version: Option<i64>,
    },

    /// Re-create your database from migrations.
    /// WARNING: this deletes all data.
    Flush {
        #[arg(
            long,
            help = "Confirm you want your database destroyed",
            default_value = "false"
        )]
        yes: bool,
    },

    /// Revert migrations
    Revert {
        #[arg(long, help = "Revert to this migration version")]
        version: Option<i64>,
    },

    /// Add a new migration
    Add {
        #[arg(long, short, help = "Migration name", default_value = "unnamed")]
        name: String,
    },
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUM_LOG_QUERIES", "1");
    Logger::init();

    if !check_root() {
        eprintln!("{}", "rwf-cli must run from the root of a cargo project",);
        std::process::exit(1);
    }

    let args = Cli::parse();

    match args.subcommands {
        Subcommands::Migrate(migrate) => match migrate.command {
            Migrate::Run { version } => migrate::migrate(version).await,
            Migrate::Revert { version } => migrate::revert(version).await,
            Migrate::Flush { yes } => {
                if yes {
                    migrate::revert(None).await;
                    migrate::migrate(None).await;
                } else {
                    log::info!("Aborting");
                }
            }
            Migrate::Add { name } => migrate::add(&name).await,
        },

        Subcommands::Setup => setup::setup().await,
    }
}

fn check_root() -> bool {
    Path::new("Cargo.toml").exists()
}
