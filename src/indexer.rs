use cosmrs::rpc::Client;
use cosmrs::rpc::HttpClient;
use tokio::time::{sleep, Duration};

use crate::args::Args;
use crate::fetch;
use crate::model;
use crate::tables;

use crate::tables::address_msg::AddressMsgRow;
use crate::tables::block::BlockRow;
use crate::tables::msg::MsgRow;
use crate::tables::tx::TxRow;

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

fn index_history_lower_bound(conn: &mut rusqlite::Connection, args: &Args) -> rusqlite::Result<u64> {
    let top = tables::block::top(&mut conn)
        .map(|it| std::cmp::max(it.height + 1, args.from_block as u64))
        .unwrap_or(args.from_block as u64);
    Ok(top)
}

async fn index_history_upper_bound(client: &HttpClient, args: &Args) -> u64 {
    client
        .latest_block()
        .await
        .map(|resp| resp.block.header.height.value())
        .map(|it| std::cmp::min(it, args.to_block as u64))
        .unwrap_or(args.to_block as u64)
}

pub async fn index_history(args: &Args) {
    let mut conn = tables::schema::conn(&args.datadir).unwrap();
    let client = &HttpClient::new(args.rpc.as_str()).unwrap();

    loop {
        let lb = index_history_lower_bound(&mut conn, args).unwrap();
        let ub = index_history_upper_bound(client, args).await;

        log::info!("Considering range : {} -> {}", lb, ub);

        for i in lb..(ub + 1) {
            index_block(&mut conn, client, i).await;

            if i % 1000 == 0 {
                log::info!("Reached block : {}", i)
            } else {
                log::debug!("Reached block : {}", i)
            }
        }
        sleep(Duration::from_millis(1000)).await
    }
}

fn index_transfers_lower_bound(conn: &mut rusqlite::Connection, args: &Args) -> rusqlite::Result<u64> {
    todo!();
}
fn index_transfers_upper_bound(conn: &mut rusqlite::Connection, args: &Args) -> rusqlite::Result<u64> {
    todo!();
}

pub async fn index_transfers(args: &Args) {
    let mut conn = tables::schema::conn(&args.datadir).unwrap();

    loop {
        let lb = index_transfers_lower_bound(&mut conn, args).unwrap();
        let ub = index_transfers_upper_bound(&mut conn, args).unwrap();

        log::info!("Considering range : {} -> {}", lb, ub);

        for i in lb..(ub + 1) {
            //index_block(&mut conn, client, i).await;

            if i % 1000 == 0 {
                log::info!("Reached block : {}", i)
            } else {
                log::debug!("Reached block : {}", i)
            }
        }
        sleep(Duration::from_millis(1000)).await
    }
}
