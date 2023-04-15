use bevy_matchbox::matchbox_socket::{Packet, PeerId};

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone)]
pub struct RecvMessageEvent(PeerId, Packet);
