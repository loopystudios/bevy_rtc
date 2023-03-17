use log::error;
use matchbox_socket::{Packet, PeerId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Payload {
    Chat { from: PeerId, message: String },
    DrawPoint { x1: f32, y1: f32, x2: f32, y2: f32 },
}

impl From<Payload> for Packet {
    fn from(value: Payload) -> Self {
        bincode::serialize(&value)
            .unwrap_or_else(|e| {
                error!("{e:?} ");
                vec![]
            })
            .into_boxed_slice()
    }
}

impl From<Packet> for Payload {
    fn from(value: Packet) -> Self {
        bincode::deserialize(&value).unwrap()
    }
}
