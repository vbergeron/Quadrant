use clap::Parser;
use cosmrs::rpc::{Client, HttpClient};
use env_logger;
use log;

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

async fn index_block(conn: &mut rusqlite::Connection, client: &HttpClient, height: u32) {
    let res = client.block(height).await.unwrap();
    let block = fetch::block_to_model(&res).unwrap();
    insert_block(conn, &block).unwrap();
}

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", args.log_level)
    }

    env_logger::init();

    log::info!("Starting Quadrant {}", VERSION);

    if args.init_schema {
        std::fs::create_dir_all(&args.datadir).unwrap();
        tables::schema::init(&args.datadir).unwrap()
    }

    let mut conn = tables::schema::conn(&args.datadir).unwrap();

    let client: Box<HttpClient> = Box::new(HttpClient::new(args.rpc.as_str()).unwrap());

    for i in args.from_block..args.to_block {
        index_block(&mut conn, &client, i).await;
    }

    let mut tx = conn.transaction().unwrap();
    let top = tables::block::top(&mut tx);
    tx.commit().unwrap();

    log::info!("{:?}", top);
}
