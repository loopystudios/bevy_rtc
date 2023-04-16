use crate::router::Message;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct DrawPoint {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Message for DrawPoint {
    fn id() -> u16 {
        1
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Chat {
    pub from: String,
    pub message: String,
}

impl Message for Chat {
    fn id() -> u16 {
        2
    }
}
