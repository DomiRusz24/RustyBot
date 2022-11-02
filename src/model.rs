use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerOperation {
    pub op: u32,
    pub d: Value,
    pub s: Option<i32>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SAck11 {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SHandshake10 {
    #[serde(rename = "heartbeat_interval")]
    pub heartbeat_interval: u32,
}

pub struct MetaMessage {
    pub op: u32,
    pub s: Option<i32>
}

impl MetaMessage {
    pub fn new(op: u32, s: Option<i32>) -> Self {
        MetaMessage { op, s }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMessages {
    None,
    Ack(SAck11),
    Handshake(SHandshake10)
}

impl ServerMessages {
    pub fn parse_server_message(msg: &str) -> (MetaMessage, Self) {
        let msg_json: ServerOperation = serde_json::from_str(msg).unwrap();

        let op_code = msg_json.op;
        let sequence = msg_json.s;
        let data = msg_json.d;
        let meta = MetaMessage::new(op_code, sequence);

        match op_code {
            10 => {
                (meta, Self::Handshake(serde_json::from_value::<SHandshake10>(data).unwrap()))
            }
            11 => {
                (meta, Self::Ack(serde_json::from_value::<SAck11>(data).unwrap()))
            }

            _ => {
                (meta, Self::None)
            }
        }
    }
}



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub op: u32,
    pub d: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Heartbeat1 {
    pub op: u32,
    pub d: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identify2 {
    pub token: String,
    pub intents: i64,
    pub properties: Properties
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub os: String,
    pub browser: String,
    pub device: String,
}

