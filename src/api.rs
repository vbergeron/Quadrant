use axum::extract::*;
use axum::routing::*;
use axum::*;
use log;
use r2d2_sqlite::SqliteConnectionManager;
use serde;

use std::sync::Arc;

use crate::args::Args;
use crate::tables;
use crate::tables::block::BlockRow;
use crate::tables::schema;
use crate::tables::tx::TxRow;

pub async fn init(args: &Args) {
    let db_path = args.datadir.join(schema::DB_NAME);

    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Arc::new(r2d2::Pool::new(manager).unwrap());

    let app = Router::new()
        .route("/block/:height", get(query_block_by_height))
        .route("/block/latest", get(query_block_latest))
        .route("/tx/:hash", get(query_tx_by_hash))
        .layer(Extension(pool));

    let server = Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service());

    log::info!("REST API started on port 3000");

    server.await.unwrap();
}

type SQLitePool = Arc<r2d2::Pool<SqliteConnectionManager>>;

#[derive(Debug, serde::Serialize)]
struct BlockView {
    height: u64,
    hash: String,
    time: String,
    proposer: String,
    tx_count: usize,
    tx_hashes: Vec<String>
}

impl BlockView {
    fn from(block:BlockRow, txs: Vec<TxRow>) -> BlockView {
        BlockView {
            height: block.height,
            hash: block.hash,
            time: block.time.to_rfc3339(),
            proposer: block.proposer,
            tx_count: txs.len(),
            tx_hashes: txs.into_iter().map(|tx| tx.hash).collect(),
        }
    }
}

async fn query_block_latest(Extension(pool): Extension<SQLitePool>) -> Json<BlockView> {
    let mut conn = pool.get().unwrap();
    let res = tables::block::top(&mut conn).unwrap();
    let txs = tables::tx::by_block(&mut conn, res.height).unwrap();
    Json(BlockView::from(res, txs))
}

async fn query_block_by_height(
    Extension(pool): Extension<SQLitePool>,
    Path(height): Path<u64>,
) -> Json<BlockView> {
    let mut conn = pool.get().unwrap();
    let res = tables::block::by_height(&mut conn, height).unwrap();
    let txs = tables::tx::by_block(&mut conn, res.height).unwrap();
    Json(BlockView::from(res, txs))
}

#[derive(Debug, serde::Serialize)]
struct TxView {
    hash: String,
    index: u32,
    block: u64,
}

impl Into<TxView> for TxRow {
    fn into(self) -> TxView {
        TxView {
            hash: self.hash,
            index: self.idx,
            block: self.block,
        }
    }
}

async fn query_tx_by_hash(
    Extension(pool): Extension<SQLitePool>,
    Path(hash): Path<String>,
) -> Json<TxView> {
    let mut conn = pool.get().unwrap();
    let res = tables::tx::by_hash(&mut conn, &hash).unwrap();
    Json(res.into())
}
