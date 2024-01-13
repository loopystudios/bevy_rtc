use bevy::prelude::Event;
use bevy_matchbox::matchbox_socket::{Packet, PeerId};

/// The raw event to receive from a socket
#[derive(Debug, Clone, Event)]
pub struct SocketRecvEvent(pub (PeerId, Packet));
