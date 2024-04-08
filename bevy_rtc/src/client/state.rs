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
    pub(crate) addr: Option<String>,
    /// The Peer ID of the host
    pub(crate) host_peer_id: Option<PeerId>,
    /// The Peer ID given by the signaling server
    pub(crate) peer_id: Option<PeerId>,
    /// The latency to the server
    pub(crate) latency: Option<Duration>,
    /// The smooth latency to the server
    pub(crate) smoothed_latency: Option<Duration>,
}

impl RtcClientState {
    /// Returns the address bound by the server/host.
    pub fn addr(&self) -> Option<&str> {
        self.addr.as_deref()
    }

    /// Returns the peer ID of this client if connected
    pub fn peer_id(&self) -> Option<PeerId> {
        self.peer_id
    }

    /// Returns the peer ID of the server if connected
    pub fn host_peer_id(&self) -> Option<PeerId> {
        self.host_peer_id
    }

    /// Return the latency to the server if connected
    pub fn latency(&self) -> Option<Duration> {
        self.latency
    }

    /// Return the smoothed latency to the server if connected
    pub fn smoothed_latency(&self) -> Option<Duration> {
        self.smoothed_latency
    }
}
