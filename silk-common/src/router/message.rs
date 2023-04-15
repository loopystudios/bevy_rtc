use bevy_matchbox::matchbox_socket::Packet;

pub trait Message:
    core::fmt::Debug + Clone + std::default::Default + Send + Sync + 'static
{
    fn id() -> u16;
    fn from_packet(packet: &Packet) -> Option<Self>;
    fn to_packet(&self) -> Packet;
}
