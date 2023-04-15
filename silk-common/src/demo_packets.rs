use crate::packets::SilkPayload;
use bevy_matchbox::matchbox_socket::Packet;
use bincode::ErrorKind;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum PaintingDemoPayload {
    Chat { from: String, message: String },
    DrawPoint { x1: f32, y1: f32, x2: f32, y2: f32 },
}

impl From<&PaintingDemoPayload> for SilkPayload {
    fn from(value: &PaintingDemoPayload) -> Self {
        SilkPayload::Message(Packet::from(value))
    }
}

impl From<&PaintingDemoPayload> for Packet {
    fn from(value: &PaintingDemoPayload) -> Self {
        bincode::serialize(&value).unwrap().into_boxed_slice()
    }
}

impl TryFrom<&Packet> for PaintingDemoPayload {
    type Error = Box<ErrorKind>;

    fn try_from(value: &Packet) -> Result<Self, Self::Error> {
        bincode::deserialize(value)
    }
}
