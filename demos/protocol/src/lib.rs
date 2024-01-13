use bevy_rtc::protocol::Payload;
use serde::{Deserialize, Serialize};

// Used by painting demo

#[derive(Payload, Serialize, Deserialize, Debug, Clone)]
pub struct DrawLinePayload {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone)]
pub struct ChatPayload {
    pub from: String,
    pub message: String,
}

// Used by ping demo

#[derive(Payload, Serialize, Deserialize, Debug, Clone)]
pub enum PingPayload {
    Ping,
    Pong,
}
