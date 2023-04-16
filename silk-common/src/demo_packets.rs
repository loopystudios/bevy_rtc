use crate::{packet::SilkPacket, router::Message};
use bevy_matchbox::matchbox_socket::Packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct DrawPoint {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Message for DrawPoint {
    fn from_packet(packet: &Packet) -> Option<DrawPoint> {
        bincode::deserialize::<SilkPacket<DrawPoint>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .and_then(|silk_packet| Some(silk_packet.message))
    }

    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            message: self.clone(),
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

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
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<Chat>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .and_then(|silk_packet| Some(silk_packet.message))
    }

    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            message: self.clone(),
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

    fn id() -> u16 {
        2
    }
}
