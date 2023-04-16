use crate::router::Message;
use serde::{Deserialize, Serialize};

pub mod auth;

#[derive(Deserialize, Serialize)]
pub struct SilkPacket<M: Message> {
    pub msg_id: u16,
    pub data: M,
}
