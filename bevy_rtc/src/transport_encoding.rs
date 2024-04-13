use bevy::{ecs::system::Resource, reflect::erased_serde::Serialize};
use bevy_matchbox::matchbox_socket::Packet;
use serde::Deserialize;

use crate::{prelude::Protocol, protocol::RtcPacket};

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum TransportEncoding {
    Json,
    #[cfg(feature = "binary")]
    #[cfg_attr(docsrs, doc(cfg(feature = "binary")))]
    Binary,
}

impl TransportEncoding {
    pub(crate) fn decode_packet<T>(&self, packet: &Packet) -> Option<T>
    where
        T: for<'a> Deserialize<'a> + Protocol,
    {
        match self {
            TransportEncoding::Json => serde_json::from_slice::<RtcPacket<T>>(packet).ok(),
            #[cfg(feature = "binary")]
            TransportEncoding::Binary => bincode::deserialize::<RtcPacket<T>>(packet).ok(),
        }
        .filter(|packet| packet.msg_id == T::id())
        .map(|packet| packet.data)
    }

    pub(crate) fn encode_packet<T>(&self, v: &T) -> Packet
    where
        T: Serialize + Protocol,
    {
        let packet = RtcPacket {
            msg_id: T::id(),
            data: v.clone(),
        };
        match self {
            TransportEncoding::Json => serde_json::to_string(&packet).unwrap().as_bytes().into(),
            #[cfg(feature = "binary")]
            TransportEncoding::Binary => bincode::serialize(&packet).unwrap().into_boxed_slice(),
        }
    }
}
