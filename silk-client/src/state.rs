use bevy::prelude::*;
use silk_common::{
    bevy_matchbox::prelude::PeerId, AuthenticationRequest, ConnectionAddr,
};

/// State of the socket
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Establishing,
    LoggingIn,
    Connected,
}

#[derive(Resource, Default)]
pub struct ClientState {
    /// The socket address, used for connecting/reconnecting
    pub addr: Option<ConnectionAddr>,
    /// The authentication used for connecting/reconnecting
    pub auth: Option<AuthenticationRequest>,
    /// The ID of the host
    pub host_id: Option<PeerId>,
    /// The ID given by the signaling server
    pub id: Option<PeerId>,
}
