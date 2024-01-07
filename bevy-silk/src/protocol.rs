use bevy_matchbox::matchbox_socket::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

// Note: Intentional name collision with the trait Payload!
// This is done commonly, like `serde::Serialize` is a trait and a derive macro.
pub use proc_macro_payload::Payload;

#[derive(Deserialize, Serialize)]
#[serde(bound = "M: DeserializeOwned")]
pub struct SilkPacket<M: Payload> {
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
        serde_json::from_slice::<SilkPacket<Self>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    #[cfg(feature = "binary")]
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<Self>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    #[cfg(not(feature = "binary"))]
    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };

        serde_json::to_string(&silk_packet)
            .unwrap()
            .as_bytes()
            .into()
    }

    #[cfg(feature = "binary")]
    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };

        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }
}

#[derive(Debug, Clone)]
pub enum AuthenticationRequest {
    Registered {
        access_token: String,
        character: String,
    },
    Guest {
        username: Option<String>,
    },
}
impl Default for AuthenticationRequest {
    fn default() -> Self {
        AuthenticationRequest::Guest { username: None }
    }
}
