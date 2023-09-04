use bevy::prelude::Event;
use bevy_matchbox::matchbox_socket::{Packet, PeerId};

#[derive(Debug, Clone, Event)]
pub struct SocketRecvEvent(pub (PeerId, Packet));

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone, Event)]
pub enum SilkClientEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// The socket has successfully connected to a host
    ConnectedToHost { host: PeerId, username: String },
    /// The socket disconnected from the host
    DisconnectedFromHost { reason: Option<String> },
}

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone, Event)]
pub enum SilkServerEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    GuestLoginRequest {
        peer_id: PeerId,
        /// Optional username
        username: Option<String>,
    },
    LoginRequest {
        peer_id: PeerId,
        access_token: String,
        character: String,
    },
    /// A peer has left this server
    ClientLeft(PeerId),
}
