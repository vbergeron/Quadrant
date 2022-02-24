use crate::fp;
use crate::model;
use rusqlite::*;

#[derive(Debug)]
pub struct AddressMsgRow {
    pub address: String,
    pub block: u64,
    pub tx: u32,
    pub msg: u32,
}

impl TryFrom<&Row<'_>> for AddressMsgRow {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(AddressMsgRow {
            address: row.get(0)?,
            block: row.get(1)?,
            tx: row.get(2)?,
            msg: row.get(3)?,
        })
    }
}

impl AddressMsgRow {
    pub fn new(block: &model::Block, tx: &model::Tx, msg: &model::Msg, address: &String) -> Self {
        AddressMsgRow {
            address: address.clone(),
            block: block.height,
            tx: tx.index,
            msg: msg.index,
        }
    }
}

const INSERT: &str = "INSERT INTO address_msg (address, block, tx, msg) VALUES (?,?,?,?)";
pub fn insert<T>(conn: &mut T, row: &AddressMsgRow) -> Result<()>
where
    T: core::ops::Deref<Target = Connection>,
{
    conn.prepare_cached(INSERT)?
        .execute(params![row.address, row.block, row.tx, row.msg])
        .map(fp::as_unit)
}
