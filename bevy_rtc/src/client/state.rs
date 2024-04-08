use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use instant::Duration;

/// State of the socket
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum RtcClientStatus {
    /// Disconnected
    #[default]
    Disconnected,
    /// Connecting
    Establishing,
    /// Connected
    Connected,
}

#[derive(Resource, Default)]
pub struct RtcClientState {
    /// The socket address, used for connecting/reconnecting
    pub addr: Option<String>,
    /// The ID of the host
    pub host_id: Option<PeerId>,
    /// The ID given by the signaling server
    pub id: Option<PeerId>,
    /// The latency to the server
    pub latency: Option<Duration>,
    /// The smooth latency to the server
    pub smoothed_latency: Option<Duration>,
}
