use crate::Packet;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExamplePacket {
    name: String,
    message: String,
}

impl From<ExamplePacket> for Packet {
    fn from(value: ExamplePacket) -> Self {
        bincode::serialize(&value)
            .unwrap_or_else(|e| {
                error!("{e:?} ");
                vec![]
            })
            .into_boxed_slice()
    }
}
