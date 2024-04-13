use bevy_matchbox::matchbox_socket::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

// Note: Intentional name collision with the trait Payload!
// This is done commonly, like `serde::Serialize` is a trait and a derive macro.
pub use proc_macro_protocol::Protocol;

use crate::transport_encoding::TransportEncoding;

#[derive(Deserialize, Serialize)]
#[serde(bound = "M: DeserializeOwned")]
pub(crate) struct RtcPacket<M: Protocol> {
    pub msg_id: u16,
    pub data: M,
}

pub trait Protocol:
    Debug + Clone + Send + Sync + for<'a> Deserialize<'a> + Serialize + 'static
{
    fn id() -> u16;

    fn reflect_name() -> &'static str;

    fn from_packet(packet: &Packet, deserializer: &TransportEncoding) -> Option<Self> {
        deserializer.decode_packet(packet)
    }

    fn to_packet(&self, serializer: &TransportEncoding) -> Packet {
        serializer.encode_packet(self)
    }
}
