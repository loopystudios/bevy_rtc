use bevy_rtc::protocol::Protocol;
use serde::{Deserialize, Serialize};

// Used by painting demo

#[derive(Protocol, Serialize, Deserialize, Debug, Clone)]
pub struct DrawLinePayload {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Protocol, Serialize, Deserialize, Debug, Clone)]
pub struct ChatPayload {
    pub from: String,
    pub message: String,
}

// Used by ping demo

#[derive(Protocol, Serialize, Deserialize, Debug, Clone)]
pub struct PingPayload;

#[derive(Protocol, Serialize, Deserialize, Debug, Clone)]
pub struct PongPayload;
