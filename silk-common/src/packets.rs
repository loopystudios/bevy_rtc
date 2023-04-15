use serde::{Deserialize, Serialize};

use crate::router::Message;

#[derive(Deserialize, Serialize)]
pub struct SilkPacket<M: Message> {
    pub msg_id: u16,
    pub message: M,
}
