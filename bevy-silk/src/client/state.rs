use crate::protocol::AuthenticationRequest;
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;

/// State of the socket
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum SilkConnectionState {
    /// Unauthenticated and inactive
    #[default]
    Disconnected,
    /// Connecting to the websocket
    Establishing,
    /// Connected to the websocket, but not authenticated by the server
    Authenticating,
    /// Authenticated and active
    Connected,
}

#[derive(Resource, Default)]
pub struct SilkState {
    /// The socket address, used for connecting/reconnecting
    pub addr: Option<String>,
    /// The authentication used for connecting/reconnecting
    pub auth: Option<AuthenticationRequest>,
    /// The ID of the host
    pub host_id: Option<PeerId>,
    /// The ID given by the signaling server
    pub id: Option<PeerId>,
}
