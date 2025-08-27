use clap::{Parser, Subcommand};
use glade::{DatabaseManager, Result};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Database {
        #[clap(subcommand)]
        action: DatabaseAction,
    },
}

#[derive(Subcommand)]
enum DatabaseAction {
    Download {
        #[clap(long, conflicts_with = "all")]
        database: Option<String>,

        #[clap(long, conflicts_with = "all")]
        genome_version: Option<String>,

        #[clap(long)]
        all: bool,
    },

    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Database { action } => {
            match action {
                DatabaseAction::Download {
                    database,
                    genome_version,
                    all,
                } => {
                    let manager = DatabaseManager::new()?;

                    if all {
                        manager.download_all_databases().await?;
                    } else if let (Some(db_name), Some(version)) = (database, genome_version) {
                        manager.download_database(&db_name, &version).await?;
                    } else {
                        eprintln!("Error: Must specify either --all or both --database and --genome-version");
                        std::process::exit(1);
                    }
                }
                DatabaseAction::List => {
                    let manager = DatabaseManager::new()?;
                    manager.list_databases()?;
                }
            }
        }
    }

    Ok(())
}
