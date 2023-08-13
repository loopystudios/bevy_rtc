use bevy_matchbox::matchbox_socket::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize, Serialize)]
#[serde(bound = "M: DeserializeOwned")]
pub struct SilkPacket<M: Payload> {
    pub msg_id: u16,
    pub data: M,
}

#[cfg(not(any(feature = "json", feature = "bincode")))]
compile_error!(
    "you must enable feature \"json\" or \"bincode\" to choose a transport format"
);

#[cfg(all(feature = "json", feature = "bincode"))]
compile_error!(
    "feature \"json\" and feature \"bincode\" cannot be enabled at the same time"
);

pub trait Payload:
    Debug + Clone + Send + Sync + for<'a> Deserialize<'a> + Serialize + 'static
{
    fn id() -> u16;

    fn reflect_name() -> &'static str;

    #[cfg(feature = "json")]
    fn from_packet(packet: &Packet) -> Option<Self> {
        serde_json::from_slice::<SilkPacket<Self>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    #[cfg(feature = "bincode")]
    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<Self>>(packet)
            .ok()
            .filter(|silk_packet| silk_packet.msg_id == Self::id())
            .map(|silk_packet| silk_packet.data)
    }

    #[cfg(feature = "json")]
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

    #[cfg(feature = "bincode")]
    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };

        bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
    }
}
