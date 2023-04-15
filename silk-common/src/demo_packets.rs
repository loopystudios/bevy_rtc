use bevy_matchbox::matchbox_socket::Packet;
use serde::{Deserialize, Serialize};

use crate::network_queries::Message;

#[derive(Serialize, Deserialize, Clone)]
pub enum PaintingDemoPayload {
    Chat { from: String, message: String },
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

#[derive(Default, Deserialize)]
pub struct Chat {
    from: String,
    message: String,
}

impl Message for Chat {}

#[derive(Default, Deserialize)]
pub struct DrawPoint {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl Message for DrawPoint {}
