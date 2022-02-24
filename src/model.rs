use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Block {
    pub hash: String,
    pub height: u64,
    pub time: DateTime<Utc>,
    pub proposer: String,
    pub txs: Vec<Tx>,
}

#[derive(Debug)]
pub struct Tx {
    pub hash: String,
    pub index: u32,
    pub msgs: Vec<Msg>,
}

#[derive(Debug)]
pub struct Msg {
    pub index: u32,
    pub tag: String,
    pub data: Vec<u8>,
    pub addresses: Vec<String>,
}
