use crate::fp;
use crate::model;
use rusqlite::*;

#[derive(Debug)]
pub struct TxRow {
    pub block: u64,
    pub idx: u32,
    pub hash: String,
}

impl TryFrom<&Row<'_>> for TxRow {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(TxRow {
            block: row.get(0)?,
            idx: row.get(1)?,
            hash: row.get(2)?,
        })
    }
}

impl TxRow {
    pub fn new(block: &model::Block, tx: &model::Tx) -> Self {
        TxRow {
            block: block.height,
            idx: tx.index,
            hash: tx.hash.clone(),
        }
    }
}

const INSERT: &str = "INSERT INTO tx (block, idx, hash) VALUES (?,?,?)";
pub fn insert<T>(conn: &mut T, row: &TxRow) -> Result<()>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(INSERT)?
        .execute(params![row.block, row.idx, row.hash])
        .map(fp::as_unit)
}

const BY_HASH: &str = "SELECT block, idx, hash FROM tx WHERE hash = ?";
pub fn by_hash<T>(conn: &mut T, hash: &String) -> Result<TxRow>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(BY_HASH)?
        .query_row(params![hash], |row| TxRow::try_from(row))
}

const BY_BLOCK: &str = "SELECT block, idx, hash FROM tx WHERE block = ?";
pub fn by_block<T>(conn: &mut T, block: u64) -> Result<Vec<TxRow>>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(BY_BLOCK)?
        .query_map(params![block], |row| TxRow::try_from(row))?
        .into_iter()
        .collect()
}
