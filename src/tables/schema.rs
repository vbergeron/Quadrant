use rusqlite::*;
use std::path::PathBuf;

use crate::args::Args;

pub const SCHEMA: &str = unsafe { std::str::from_utf8_unchecked(include_bytes!("schema.sql")) };

pub const DB_NAME: &str = "data.db";

pub fn conn(datadir: &PathBuf) -> Result<Connection> {
    let db = datadir.join(DB_NAME);
    Connection::open(db)
}

pub fn init(args: &Args) -> Result<()> {
    std::fs::create_dir_all(&args.datadir).unwrap();

    let conn = conn(&args.datadir)?;

    let stmts = SCHEMA.split(";").map(str::trim).filter(|s| !s.is_empty());

    for sql in stmts {
        conn.execute(sql, [])?;
    }

    log::info!("Initialized new database schema in data.db");
    Ok(())
}
