use bevy::{
    ecs::schedule::States,
    prelude::Resource,
    utils::{hashbrown::HashMap, HashSet},
};
use bevy_matchbox::prelude::PeerId;
use instant::Duration;
use std::net::SocketAddr;

/// State of the server
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum RtcServerStatus {
    /// Not ready
    #[default]
    NotReady,
    /// Ready
    Ready,
}

#[derive(Resource)]
pub struct RtcState {
    /// The socket address bound
    pub addr: SocketAddr,

    /// The ID the host (server)
    pub id: Option<PeerId>,

    /// A list of connected peers
    pub(crate) peers: HashSet<PeerId>,

    /// A map of user latencies
    pub(crate) latencies: HashMap<PeerId, Option<Duration>>,

    /// A map of smoothed user latencies
    pub(crate) smoothed_latencies: HashMap<PeerId, Option<Duration>>,
}

impl RtcState {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            id: None,
            peers: HashSet::new(),
            latencies: HashMap::new(),
            smoothed_latencies: HashMap::new(),
        }
    }

    /// Return the currently connected peers
    pub fn peers(&self) -> impl Iterator<Item = PeerId> + '_ {
        self.peers.iter().copied()
    }

    /// Return the instantaneous latencies for all peers
    pub fn iter_latencies(&self) -> impl Iterator<Item = (PeerId, Duration)> + '_ {
        self.latencies
            .iter()
            .filter_map(|(p, l)| l.map(|l| (p, l)))
            .map(|(p, l)| (*p, l))
    }

    /// Return the smoothed latencies for all peers
    pub fn iter_smoothed_latencies(&self) -> impl Iterator<Item = (PeerId, Duration)> + '_ {
        self.smoothed_latencies
            .iter()
            .filter_map(|(p, l)| l.map(|l| (p, l)))
            .map(|(p, l)| (*p, l))
    }

    /// Return the latency for a peer if they exist
    pub fn get_latency_for(&self, peer_id: PeerId) -> Option<Duration> {
        *self.latencies.get(&peer_id)?
    }

    /// Return the smoothed latency for a peer if they exist
    pub fn get_smoothed_latency_for(&self, peer_id: PeerId) -> Option<Duration> {
        *self.smoothed_latencies.get(&peer_id)?
    }
}
