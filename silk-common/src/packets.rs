use crate::Packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExamplePacket {
    name: String,
    message: String,
}

impl From<ExamplePacket> for Packet {
    fn from(value: ExamplePacket) -> Self {
        bincode::serialize(&value).unwrap().into_boxed_slice()
    }
}
