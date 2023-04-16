use proc_macro_payload::Payload;
use serde::{Deserialize, Serialize};

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct DrawPoint {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct Chat {
    pub from: String,
    pub message: String,
}
