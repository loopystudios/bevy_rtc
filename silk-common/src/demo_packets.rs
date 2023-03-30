use bevy_matchbox::matchbox_socket::{Packet, PeerId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum PaintingDemoPayload {
    Chat { from: PeerId, message: String },
    DrawPoint { x1: f32, y1: f32, x2: f32, y2: f32 },
}

impl From<PaintingDemoPayload> for Packet {
    fn from(value: PaintingDemoPayload) -> Self {
        bincode::serialize(&value).unwrap().into_boxed_slice()
    }
}

impl From<Packet> for PaintingDemoPayload {
    fn from(value: Packet) -> Self {
        bincode::deserialize(&value).unwrap()
    }
}
