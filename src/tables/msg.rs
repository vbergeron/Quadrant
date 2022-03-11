use crate::fp;
use crate::model;
use rusqlite::*;

#[derive(Debug)]
pub struct MsgRow {
    pub block: u64,
    pub tx: u32,
    pub idx: u32,
    pub tag: String,
    pub data: Vec<u8>,
}

impl TryFrom<&Row<'_>> for MsgRow {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(MsgRow {
            block: row.get(0)?,
            tx: row.get(1)?,
            idx: row.get(2)?,
            tag: row.get(3)?,
            data: row.get(4)?,
        })
    }
}

impl MsgRow {
    pub fn new(block: &model::Block, tx: &model::Tx, msg: &model::Msg) -> Self {
        MsgRow {
            block: block.height,
            tx: tx.index,
            idx: msg.index,
            tag: msg.tag.clone(),
            data: msg.data.clone(),
        }
    }
}

const INSERT: &str = "INSERT INTO msg (block, tx, idx, tag, data) VALUES (?,?,?,?,?)";
pub fn insert<T>(conn: &mut T, row: &MsgRow) -> Result<()>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(INSERT)?
        .execute(params![row.block, row.tx, row.idx, row.tag, row.data])
        .map(fp::as_unit)
}

const BY_BLOCK: &str = "SELECT (block, tx, idx, tag, data) FROM msg WHERE block = ?";
pub fn by_block<T>(conn: &mut T, block: u64) -> Result<Vec<MsgRow>>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(BY_BLOCK)?
        .query_map(params![block], |row| MsgRow::try_from(row))?
        .into_iter()
        .collect()
}
