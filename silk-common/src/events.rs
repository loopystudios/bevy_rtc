use bevy_matchbox::matchbox_socket::{Packet, PeerId};

#[derive(Debug, Clone)]
pub struct RecvMessageEvent(pub PeerId, pub Packet);

#[derive(Debug, Clone)]
pub struct SendMessageEvent(pub PeerId, pub Packet);
