use bevy_matchbox::matchbox_socket::Packet;
use log::{error, warn};
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
        let result: Result<SilkPacket<Self>, serde_json::Error> =
            serde_json::from_slice(packet);
        match result {
            Ok(packet) => {
                if packet.msg_id == Self::id() {
                    Some(packet.data)
                } else {
                    None
                }
            }
            Err(e) => {
                error!("failed to deserialize packet! err: {e:?}, bytes: {packet:?}");
                None
            }
        }
        //bincode::deserialize::<SilkPacket<Self>>(packet)
        //    .ok()
        //    .filter(|silk_packet| silk_packet.msg_id == Self::id())
        //    .map(|silk_packet| silk_packet.data)
    }

    fn to_packet(&self) -> Packet {
        let silk_packet = SilkPacket {
            msg_id: Self::id(),
            data: self.clone(),
        };

        let input_string = serde_json::to_string(&silk_packet).unwrap();

        // Convert the string to bytes
        let bytes = input_string.as_bytes();

        // Allocate a Box to hold the bytes
        let boxed_bytes: Box<[u8]> = bytes.into();
        //bincode::serialize(&silk_packet).unwrap().into_boxed_slice()
        boxed_bytes
    }
}
