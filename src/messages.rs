#[derive(Debug)]
struct CosmosMsgTransfer {
    sender: AccountId,
    receiver: AccountId,
    port: String,
    channel: String,
    amount: Option<Decimal>,
}

impl From<ibc::applications::transfer::v1::MsgTransfer> for CosmosMsgTransfer {
    fn from(msg: ibc::applications::transfer::v1::MsgTransfer) -> Self {
        CosmosMsgTransfer {
            sender: msg.sender.parse().unwrap(),
            receiver: msg.receiver.parse().unwrap(),
            port: msg.source_port,
            channel: msg.source_channel,
            amount: msg.token.map(|t| t.amount.parse().unwrap()),
        }
    }
}

#[derive(Debug)]
struct CosmosMsgSend {
    from: AccountId,
    to: AccountId,
    amount: Decimal,
}

impl From<cosmrs::bank::MsgSend> for CosmosMsgSend {
    fn from(msg: cosmrs::bank::MsgSend) -> Self {
        CosmosMsgSend {
            from: msg.from_address,
            to: msg.to_address,
            amount: msg.amount[0].amount,
        }
    }
}

#[derive(Debug)]
enum CosmosMsg {
    Send(CosmosMsgSend),
    Transfer(CosmosMsgTransfer),
    Raw(Any),
}

fn parse_msg(msg: &Any) -> Result<CosmosMsg, cosmrs::ErrorReport> {
    match msg.type_url.as_str() {
        "/cosmos.bank.v1beta1.MsgSend" => {
            let parsed = cosmrs::bank::MsgSend::from_any(msg)?;
            Ok(CosmosMsg::Send(CosmosMsgSend::from(parsed)))
        }
        "/ibc.applications.transfer.v1.MsgTransfer" => {
            let parsed = ibc::applications::transfer::v1::MsgTransfer::decode(&msg.value[..])?;
            Ok(CosmosMsg::Transfer(CosmosMsgTransfer::from(parsed)))
        }
        _ => Ok(CosmosMsg::Raw(msg.to_owned())),
    }
}