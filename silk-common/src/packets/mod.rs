use crate::router::Message;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod auth;

#[derive(Deserialize, Serialize)]
#[serde(bound = "M: DeserializeOwned")]
pub struct SilkPacket<M: Message> {
    pub msg_id: u16,
    pub data: M,
}
