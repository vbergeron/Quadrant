use crate::model;
use chrono::{DateTime, Utc};
use cosmrs::proto::*;
use cosmrs::tx::Msg;
use cosmrs::{rpc, tendermint::abci::Transaction, Any};
use prost::Message;
use sha2::{Digest, Sha256};

fn block_time(ts: cosmrs::tendermint::Time) -> DateTime<Utc> {
    // Unwrap is Ok here because the outputed string is well-formed
    DateTime::parse_from_rfc3339(&ts.to_rfc3339())
        .unwrap()
        .with_timezone(&Utc)
}

fn tx_hash(tx: &Transaction) -> String {
    let mut digest = Sha256::new();
    digest.update(tx.as_bytes());
    hex::encode_upper(digest.finalize())
}

const MULTI: &'static str = "MULTI";

fn msg_transfers(index: u32, msg: &Any) -> cosmrs::Result<Vec<model::Transfer>> {
    match msg.type_url.as_str() {
        "/cosmos.bank.v1beta1.MsgSend" => {
            let parsed = cosmrs::bank::MsgSend::from_any(msg)?;
            Ok(vec![
                model::Transfer { index, sender: parsed.from_address, receiver: parsed.to_address, amount: 372}
            ])
        }
        // TODO pr to cosmrs
        "/cosmos.bank.v1beta1.MsgMultiSend" => {
            let parsed = cosmos::bank::v1beta1::MsgMultiSend::decode(&msg.value[..])?;
            let mut addresses = Vec::<String>::new();
            for i in parsed.inputs {
                addresses.push(i.address);
            }
            for o in parsed.outputs {
                addresses.push(o.address);
            }

            Ok(addresses)
        }
        _ => {
            Ok(vec![])
        }
    }
}

fn msg_addresses(msg: &Any) -> cosmrs::Result<Vec<String>> {
    match msg.type_url.as_str() {
        "/cosmos.bank.v1beta1.MsgSend" => {
            let parsed = cosmrs::bank::MsgSend::from_any(msg)?;
            Ok(vec![
                parsed.from_address.to_string(),
                parsed.to_address.to_string(),
            ])
        }
        // TODO pr to cosmrs
        "/cosmos.bank.v1beta1.MsgMultiSend" => {
            let parsed = cosmos::bank::v1beta1::MsgMultiSend::decode(&msg.value[..])?;
            let mut addresses = Vec::<String>::new();
            for i in parsed.inputs {
                addresses.push(i.address);
            }
            for o in parsed.outputs {
                addresses.push(o.address);
            }

            Ok(addresses)
        }
        "/cosmos.staking.v1beta1.MsgDelegate" => {
            let parsed = cosmrs::staking::MsgDelegate::from_any(msg)?;
            Ok(vec![
                parsed.delegator_address.to_string(),
                parsed.validator_address.to_string(),
            ])
        }
        "/cosmos.staking.v1beta1.MsgUndelegate" => {
            let parsed = cosmrs::staking::MsgUndelegate::from_any(msg)?;
            Ok(vec![
                parsed.delegator_address.to_string(),
                parsed.validator_address.to_string(),
            ])
        }
        "/cosmos.staking.v1beta1.MsgBeginRedelegate" => {
            let parsed = cosmrs::staking::MsgBeginRedelegate::from_any(msg)?;
            Ok(vec![
                parsed.delegator_address.to_string(),
                parsed.validator_src_address.to_string(),
                parsed.validator_dst_address.to_string(),
            ])
        }
        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward" => {
            let parsed = cosmrs::distribution::MsgWithdrawDelegatorReward::from_any(msg)?;
            Ok(vec![
                parsed.delegator_address.to_string(),
                parsed.validator_address.to_string(),
            ])
        }
        "/cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission" => {
            let parsed = cosmrs::distribution::MsgWithdrawValidatorCommission::from_any(msg)?;
            Ok(vec![parsed.validator_address.to_string()])
        }
        "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress" => {
            let parsed = cosmrs::distribution::MsgSetWithdrawAddress::from_any(msg)?;
            Ok(vec![
                parsed.delegator_address.to_string(),
                parsed.withdraw_address.to_string(),
            ])
        }
        "/cosmos.distribution.v1beta1.MsgFundCommunityPool" => {
            let parsed = cosmrs::distribution::MsgFundCommunityPool::from_any(msg)?;
            Ok(vec![parsed.depositor.to_string()])
        }
        tag => {
            log::warn!("Unknown type URL : {}", tag);
            Ok(vec![])
        }
    }
}

fn msg_to_model(index: u32, msg: &Any) -> cosmrs::Result<model::Msg> {
    Ok(model::Msg {
        index: index,
        tag: msg.type_url.clone(),
        data: msg.value.clone(),
        addresses: msg_addresses(msg)?,
    })
}

fn tx_to_model(index: u32, tx: &Transaction) -> cosmrs::Result<model::Tx> {
    let parsed = cosmrs::Tx::from_bytes(tx.as_bytes())?;

    let mut msgs = Vec::<model::Msg>::new();
    for (i, msg) in parsed.body.messages.iter().enumerate() {
        msgs.push(msg_to_model(i as u32, msg)?)
    }

    Ok(model::Tx {
        index: index,
        hash: tx_hash(tx),
        msgs: msgs,
    })
}

pub fn block_to_model(resp: &rpc::endpoint::block::Response) -> cosmrs::Result<model::Block> {
    let mut txs = Vec::<model::Tx>::new();
    for (i, tx) in resp.block.data.iter().enumerate() {
        txs.push(tx_to_model(i as u32, tx)?)
    }

    Ok(model::Block {
        hash: resp.block_id.hash.to_string(),
        height: resp.block.header.height.value(),
        time: block_time(resp.block.header.time),
        proposer: resp.block.header.proposer_address.to_string(),
        txs: txs,
    })
}
