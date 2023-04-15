use crate::{packets::SilkPacket, router::Message};
use bevy_matchbox::matchbox_socket::Packet;
use serde::{Deserialize, Serialize};

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

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct DrawPointMessage {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl Message for DrawPointMessage {
    fn from_packet(packet: &Packet) -> Option<DrawPointMessage> {
        bincode::deserialize::<SilkPacket<DrawPointMessage>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .and_then(|silk_packet| Some(silk_packet.message))
    }

    fn to_packet(self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            message: self,
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

    fn id() -> u16 {
        1
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Chat {
    from: String,
    message: String,
}

impl Message for Chat {
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize(packet).ok()
    }

    fn to_packet(self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            message: self,
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

    fn id() -> u16 {
        2
    }
}
