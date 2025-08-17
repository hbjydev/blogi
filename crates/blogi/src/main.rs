use std::net::SocketAddr;

use anyhow::Result;
use blogi_db::{pg::PostgresDatastore, Datastore};
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Parser)]
#[clap(name = "Blogi", version = "0.1.0", about = "An atproto blogging platform")]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    /// The URL of the database to connect to
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

#[derive(Subcommand)]
enum Command {
    /// Start the API server
    Api {
        #[arg(long, short, default_value = "0.0.0.0:8000")]
        bind_addr: SocketAddr,
    },

    /// Start the ingester
    Ingester {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(EnvFilter::from_default_env())
        )
        .try_init()?;

    let db = PostgresDatastore::open(&cli.database_url).await?;

    match cli.command {
        Command::Api { bind_addr } => {
            blogi_api::start(bind_addr, db.boxed()).await
        },

        Command::Ingester {} => {
            blogi_ingester::start(db.boxed()).await
        },
    }
}
