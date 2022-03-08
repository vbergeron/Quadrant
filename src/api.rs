use axum::extract::*;
use axum::routing::*;
use axum::*;
use log;
use r2d2_sqlite::SqliteConnectionManager;

use std::sync::Arc;

use crate::tables;
use crate::args::Args;

pub async fn init(args: &Args) {

    let manager = SqliteConnectionManager::file(&args.datadir);
    let pool = Arc::new(r2d2::Pool::new(manager).unwrap());

    let app = Router::new()
        .route("/block/latest", get(block_latest))
        .layer(Extension(pool));

    let server = Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service());

    log::info!("REST API started on port 3000");

    server.await.unwrap();
}

type SQLitePool = Arc<r2d2::Pool<SqliteConnectionManager>>;

async fn block_latest(Extension(pool): Extension<SQLitePool>) -> BlockView {
    let mut conn = pool.get().unwrap();
    let res = tables::block::top(&mut conn).unwrap();
    BlockView {}
}

#[derive(Debug, serde::Serialize)]
struct BlockView {}
