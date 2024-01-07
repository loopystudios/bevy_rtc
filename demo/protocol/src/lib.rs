use bevy_silk::protocol::Payload;
use serde::{Deserialize, Serialize};

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DrawLinePayload {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChatPayload {
    pub from: String,
    pub message: String,
}
