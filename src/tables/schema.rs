use std::path::PathBuf;

use rusqlite::*;

const SCHEMA: &str = unsafe { std::str::from_utf8_unchecked(include_bytes!("schema.sql")) };

const DB_NAME: &str = "data.db";

pub fn conn(datadir: &PathBuf) -> Result<Connection> {
    let db = datadir.join(DB_NAME);
    Connection::open(db)
}

pub fn init(datadir: &PathBuf) -> Result<()> {
    let conn = conn(datadir)?;

    let stmts = SCHEMA.split(";").map(str::trim).filter(|s| !s.is_empty());

    for sql in stmts {
        conn.execute(sql, [])?;
    }

    log::info!("Initialized new database schema in data.db");
    Ok(())
}
