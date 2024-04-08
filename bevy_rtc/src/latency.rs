use bevy::prelude::*;
use bevy_matchbox::matchbox_socket::PeerId;
use instant::Duration;
use serde::{Deserialize, Serialize};

// A name import hack to ensure the Payload macro works correctly.
mod bevy_rtc {
    pub use crate::protocol;
}

/// A packet containing information to track a peer's latency
#[derive(proc_macro_protocol::Protocol, Serialize, Deserialize, Debug, Clone)]
pub struct LatencyTracerPayload {
    pub from: PeerId,
    pub sent: f64,
}

impl LatencyTracerPayload {
    pub fn new(from: PeerId) -> Self {
        Self {
            from,
            sent: instant::now(),
        }
    }

    pub fn age(&self) -> Duration {
        let time_shift = (instant::now() - self.sent).max(0.0) as u64;
        Duration::from_millis(time_shift)
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LatencyTracer {
    /// the peer being tracked
    pub peer_id: PeerId,
    /// latency network history
    pub(crate) latency_hist: Vec<(LatencyTracerPayload, Duration)>,
    /// last calculated latency (median over 3 seconds)
    pub(crate) last_latency: Option<f32>,
}

impl LatencyTracer {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            latency_hist: vec![],
            last_latency: None,
        }
    }

    /// Process a payload that came back
    pub fn process(&mut self, payload: LatencyTracerPayload) {
        let latency = payload.age() / 2;
        self.latency_hist.push((payload, latency));
    }

    /// When called, all stale latencies are pruned and this player's median
    /// latency is chosen as the player's latency. This should be called
    /// routinely, 1x per tick, so the work required to calculated this is
    /// somewhat constant and negligible.
    pub fn update_latency(&mut self) {
        // Remove stale data
        const NET_HISTORY_SECS: Duration = Duration::from_secs(3);
        self.latency_hist
            .retain(|(p, _)| p.age() <= NET_HISTORY_SECS);

        // Set to median latency
        self.latency_hist.sort_by(|(a, _), (b, _)| {
            a.sent
                .partial_cmp(&b.sent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mid = self.latency_hist.len() / 2;
        let median = self.latency_hist.get(mid).map(|(_, lat)| lat.as_secs_f32());
        self.last_latency = median;
    }
}
