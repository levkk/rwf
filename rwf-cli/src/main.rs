use clap::{Args, Parser, Subcommand};
use rwf::logging::Logger;
use rwf::model::Pool;

use std::path::{Path, PathBuf};

mod add;
mod deploy;
mod logging;
mod migrate;
mod remove;
mod setup;
mod util;

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

    /// Add a controller/view/model/all of the above
    Add(AddSubcommand),

    /// Remove a controller/view/model/all of the above
    Remove(RemoveSubcommand),

    /// Package the application into a tarball.
    Package {
        #[arg(
            long,
            short,
            help = "Path to rwf.toml config file to be included in the build"
        )]
        config: Option<PathBuf>,

        #[arg(long, short, help = "Target CPU architecture")]
        target: Option<String>,
    },
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

#[derive(Args, Debug)]
struct AddSubcommand {
    #[command(subcommand)]
    command: Add,

    #[arg(long, short, help = "Overwrite if file exists")]
    overwrite: bool,
}

#[derive(Args, Debug)]
struct RemoveSubcommand {
    #[command(subcommand)]
    command: Remove,
}

#[derive(Subcommand, Debug)]
enum Add {
    /// Create new controller.
    Controller {
        #[arg(long, short, help = "Create new controller")]
        name: String,

        #[arg(long, short, help = "Create a page controller")]
        page: bool,
    },
}

#[derive(Subcommand, Debug)]
enum Remove {
    // Create new controller.
    Controller {
        #[arg(long, short, help = "Create new controller")]
        name: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // std::env::set_var("RWF_LOG_QUERIES", "1");
    Logger::init();

    if !check_root() {
        eprintln!("rwf-cli must run from the root of a cargo project",);
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
                    let mut conn = Pool::connection()
                        .await
                        .expect("failed to get connection from pool");
                    conn.query_cached("TRUNCATE TABLE rwf_jobs", &[])
                        .await
                        .expect("failed to clean up jobs");
                    log::info!("Deleted all background jobs");
                } else {
                    log::info!("Aborting");
                }
            }
            Migrate::Add { name } => migrate::add(&name).await,
        },

        Subcommands::Setup => setup::setup().await,

        Subcommands::Add(add) => match add.command {
            Add::Controller { name, page } => {
                add::controller(&name, page, add.overwrite).await;
            }
        },

        Subcommands::Remove(remove) => match remove.command {
            Remove::Controller { name } => {
                remove::controller(&name).await.unwrap();
            }
        },

        Subcommands::Package { config, target } => deploy::package(config, target).await.unwrap(),
    }
}

fn check_root() -> bool {
    Path::new("Cargo.toml").exists()
}
