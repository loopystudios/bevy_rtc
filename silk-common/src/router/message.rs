use bevy_matchbox::matchbox_socket::Packet;
use serde::Deserialize;

pub trait Message:
    for<'a> Deserialize<'a> + std::default::Default + Send + Sync + 'static
{
    fn from_packet(packet: &Packet) -> Option<Self>;
}
