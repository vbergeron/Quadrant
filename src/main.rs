use args::Args;
use clap::Parser;
use cosmrs::rpc::{Client, HttpClient};
use env_logger;
use log;
use tokio::time::{sleep, Duration};

pub mod args;
pub mod fetch;
pub mod fp;
pub mod model;
pub mod tables;

use tables::address_msg::AddressMsgRow;
use tables::block::BlockRow;
use tables::msg::MsgRow;
use tables::tx::TxRow;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn insert_block(conn: &mut rusqlite::Connection, block: &model::Block) -> rusqlite::Result<()> {
    let mut txn = conn.transaction()?;

    let row = &BlockRow::new(block);
    tables::block::insert(&mut txn, row)?;

    for tx in &block.txs {
        let row = &TxRow::new(block, tx);
        tables::tx::insert(&mut txn, row)?;

        for msg in &tx.msgs {
            let row = &MsgRow::new(block, tx, msg);
            tables::msg::insert(&mut txn, row)?;

            for address in &msg.addresses {
                let row = AddressMsgRow::new(&block, &tx, &msg, address);
                tables::address_msg::insert(&mut txn, &row)?;
            }
        }
    }

    txn.commit()
}

async fn index_block(conn: &mut rusqlite::Connection, client: &HttpClient, height: u64) {
    let res = client.block(height as u32).await.unwrap();
    let block = fetch::block_to_model(&res).unwrap();
    insert_block(conn, &block).unwrap();
}

fn index_lower_bound(conn: &mut rusqlite::Connection, args: &Args) -> rusqlite::Result<u64> {
    let mut tx = conn.transaction()?;
    let top = tables::block::top(&mut tx)
        .map(|it| std::cmp::max(it.height + 1, args.from_block as u64))
        .unwrap_or(args.from_block as u64);
    tx.commit()?;
    Ok(top)
}

async fn index_upper_bound(client: &HttpClient, args: &Args) -> u64 {
    client.latest_block().await
        .map(|resp| resp.block.header.height.value())
        .map(|it| std::cmp::min(it, args.to_block as u64))
        .unwrap_or(args.to_block as u64)
}

async fn index_history(conn: &mut rusqlite::Connection, client: &HttpClient, args: &Args) {

    loop {
        let lb = index_lower_bound(conn, args).unwrap();
        let ub = index_upper_bound(client, args).await;

        log::info!("Considering range : {} -> {}", lb, ub);

        for i in lb..(ub + 1) {
            index_block(conn, client, i).await;

            if i % 1000 == 0 {
                log::info!("Reached block : {}", i)
            } else {
                log::debug!("Reached block : {}", i)
            }
        }
        sleep(Duration::from_millis(1000)).await
    }

}

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", &args.log_level)
    }

    env_logger::init();

    log::info!("Starting Quadrant {}", VERSION);

    if args.init_schema {
        tables::schema::init(&args).unwrap()
    }

    let mut conn = tables::schema::conn(&args.datadir).unwrap();

    let client: Box<HttpClient> = Box::new(HttpClient::new(args.rpc.as_str()).unwrap());

    let history = tokio::spawn(async move {
        index_history(&mut conn, &client, &args).await
    });

    history.await.unwrap();
}
