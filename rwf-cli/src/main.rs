use clap::{Args, Parser, Subcommand};
use rwf::logging::Logger;

mod migrate;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Migrate(MigrateSubcommand),
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
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUM_LOG_QUERIES", "1");
    Logger::init();
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
        },
    }
}
