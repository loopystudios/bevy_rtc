use bevy_matchbox::matchbox_socket::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

// Note: Intentional name collision with the trait Payload!
// This is done commonly, like `serde::Serialize` is a trait and a derive macro.
pub use proc_macro_payload::Payload;

#[derive(Deserialize, Serialize)]
#[serde(bound = "M: DeserializeOwned")]
pub struct RtcPacket<M: Payload> {
    pub msg_id: u16,
    pub data: M,
}

pub trait Payload:
    Debug + Clone + Send + Sync + for<'a> Deserialize<'a> + Serialize + 'static
{
    fn id() -> u16;

    fn reflect_name() -> &'static str;

    #[cfg(not(feature = "binary"))]
    fn from_packet(packet: &Packet) -> Option<Self> {
        serde_json::from_slice::<RtcPacket<Self>>(packet)
            .ok()
            .filter(|packet| packet.msg_id == Self::id())
            .map(|packet| packet.data)
    }

    #[cfg(feature = "binary")]
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<RtcPacket<Self>>(packet)
            .ok()
            .filter(|packet| packet.msg_id == Self::id())
            .map(|packet| packet.data)
    }

    #[cfg(not(feature = "binary"))]
    fn to_packet(&self) -> Packet {
        let packet = RtcPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };

        serde_json::to_string(&packet).unwrap().as_bytes().into()
    }

    #[cfg(feature = "binary")]
    fn to_packet(&self) -> Packet {
        let packet = RtcPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };

        bincode::serialize(&packet).unwrap().into_boxed_slice()
    }
}
