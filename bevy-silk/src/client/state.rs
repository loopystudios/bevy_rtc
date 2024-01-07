use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;

/// State of the socket
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum SilkConnectionState {
    /// Disconnected
    #[default]
    Disconnected,
    /// Connecting
    Establishing,
    /// Connected
    Connected,
}

#[derive(Resource, Default)]
pub struct SilkState {
    /// The socket address, used for connecting/reconnecting
    pub addr: Option<String>,
    /// The ID of the host
    pub host_id: Option<PeerId>,
    /// The ID given by the signaling server
    pub id: Option<PeerId>,
}
