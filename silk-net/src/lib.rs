use bevy_matchbox::matchbox_socket::Packet;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

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

    fn from_packet(packet: &Packet) -> Option<Self> {
        bincode::deserialize::<SilkPacket<Self>>(packet)
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
}
