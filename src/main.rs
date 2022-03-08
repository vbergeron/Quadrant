use clap::Parser;
use std::sync::Arc;
use env_logger;
use log;

pub mod api;
pub mod args;
pub mod fetch;
pub mod fp;
pub mod indexer;
pub mod model;
pub mod tables;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let args = Arc::new(args::Args::parse());

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", &args.log_level)
    }

    env_logger::init();

    log::info!("Starting Quadrant {}", VERSION);

    if args.init_schema {
        tables::schema::init(&args).unwrap()
    }

    let indexer_args = args.clone();
    let indexer = tokio::spawn(async move { indexer::index_history(&indexer_args).await });

    let api_args = args.clone();
    let api = tokio::spawn(async move { api::init(&api_args).await });

    indexer.await.unwrap();
    api.await.unwrap();
}
