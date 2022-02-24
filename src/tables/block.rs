use crate::fp;
use crate::model;
use chrono::{DateTime, Utc};
use rusqlite::*;

#[derive(Debug)]
pub struct BlockRow {
    pub height: u64,
    pub hash: String,
    pub time: DateTime<Utc>,
    pub proposer: String,
}

impl TryFrom<&Row<'_>> for BlockRow {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(BlockRow {
            height: row.get(0)?,
            hash: row.get(1)?,
            time: row.get(2)?,
            proposer: row.get(3)?,
        })
    }
}

impl BlockRow {
    pub fn new(block: &model::Block) -> Self {
        BlockRow {
            height: block.height,
            hash: block.hash.clone(),
            time: block.time,
            proposer: block.proposer.clone(),
        }
    }
}

const INSERT: &str = "INSERT INTO block (height, hash, time, proposer) VALUES (?,?,?,?)";
pub fn insert<T>(conn: &mut T, row: &BlockRow) -> Result<()>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(INSERT)?
        .execute(params![row.height, row.hash, row.time, row.proposer])
        .map(fp::as_unit)
}

const BY_HASH: &str = "SELECT height, hash, time, proposer FROM block WHERE hash = ?";
pub fn by_hash<T>(conn: &mut T, hash: &String) -> Result<BlockRow>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(BY_HASH)?
        .query_row(params![hash], |row| BlockRow::try_from(row))
}

const BY_HEIGHT: &str = "SELECT height, hash, time, proposer FROM block WHERE height = ?";
pub fn by_height<T>(conn: &mut T, height: u64) -> Result<BlockRow>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(BY_HEIGHT)?
        .query_row(params![height], |row| BlockRow::try_from(row))
}

const TOP_BLOCK: &str =
    "SELECT height, hash, time, proposer FROM block ORDER BY height DESC LIMIT 1";
pub fn top<T>(conn: &mut T) -> Result<BlockRow>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(TOP_BLOCK)?
        .query_row([], |row| BlockRow::try_from(row))
}
