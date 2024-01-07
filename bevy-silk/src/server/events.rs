use bevy::ecs::event::Event;
use bevy_matchbox::matchbox_socket::PeerId;

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
