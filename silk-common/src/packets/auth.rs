use crate::router::Message;
use bevy_matchbox::matchbox_socket::Packet;
use serde::{Deserialize, Serialize};

use super::SilkPacket;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkAuthUserPayload {
    pub username: String,
    pub password: String,
    pub mfa: Option<String>,
}

impl Message for SilkAuthUserPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<SilkAuthUserPayload>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

    fn id() -> u16 {
        100
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkAuthGuestPayload {
    pub username: Option<String>,
}

impl Message for SilkAuthGuestPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<SilkAuthGuestPayload>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

    fn id() -> u16 {
        101
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkLoginAcceptedPayload {
    pub username: String,
}

impl Message for SilkLoginAcceptedPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<SilkLoginAcceptedPayload>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

    fn id() -> u16 {
        102
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkLoginDeniedPayload {
    pub reason: Option<String>,
}

impl Message for SilkLoginDeniedPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<SilkLoginDeniedPayload>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };
        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }

    fn id() -> u16 {
        103
    }
}
