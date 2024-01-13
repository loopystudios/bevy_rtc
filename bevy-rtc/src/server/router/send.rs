use crate::{
    protocol::Payload,
    socket::{RtcSocket, RELIABLE_CHANNEL_INDEX, UNRELIABLE_CHANNEL_INDEX},
};
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;

#[derive(Default, Debug, Resource)]
pub struct OutgoingMessages<M: Payload> {
    pub reliable_to_all: Vec<M>,
    pub unreliable_to_all: Vec<M>,
    pub reliable_to_all_except: Vec<(PeerId, M)>,
    pub unreliable_to_all_except: Vec<(PeerId, M)>,
    pub reliable_to_peer: Vec<(PeerId, M)>,
    pub unreliable_to_peer: Vec<(PeerId, M)>,
}

impl<M: Payload> OutgoingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general,
    /// this should be called once per frame/update.
    pub fn flush(&mut self) {
        self.reliable_to_all.clear();
        self.unreliable_to_all.clear();
        self.reliable_to_all_except.clear();
        self.unreliable_to_all_except.clear();
        self.reliable_to_peer.clear();
        self.unreliable_to_peer.clear();
    }

    pub(crate) fn send_payloads(
        mut queue: ResMut<Self>,
        mut socket: ResMut<RtcSocket>,
    ) {
        // Server is sending
        for message in queue.reliable_to_all.iter() {
            let peers: Vec<PeerId> = socket.connected_peers().collect();
            peers.into_iter().for_each(|peer| {
                if socket
                    .channel_mut(RELIABLE_CHANNEL_INDEX)
                    .try_send(message.to_packet(), peer)
                    .is_err()
                {
                    error!(
                        "failed to send reliable packet to {peer}: {message:?}"
                    );
                }
            })
        }
        if !queue.reliable_to_all.is_empty() {
            trace!(
                "sent {} [R;N] {} packets",
                queue.reliable_to_all.len(),
                M::reflect_name()
            );
        }
        for message in queue.unreliable_to_all.iter() {
            let peers: Vec<PeerId> = socket.connected_peers().collect();
            peers.into_iter().for_each(|peer| {
                    if socket
                        .channel_mut(UNRELIABLE_CHANNEL_INDEX)
                        .try_send(message.to_packet(), peer).is_err() {
                        error!("failed to send unreliable packet to {peer}: {message:?}");
                    }
                })
        }
        if !queue.unreliable_to_all.is_empty() {
            trace!(
                "sent {} [U;N] {} packets",
                queue.unreliable_to_all.len(),
                M::reflect_name()
            );
        }
        for (peer, message) in queue.reliable_to_all_except.iter() {
            let peers: Vec<PeerId> =
                socket.connected_peers().filter(|p| p != peer).collect();
            peers.into_iter().for_each(|peer| {
                if socket
                    .channel_mut(RELIABLE_CHANNEL_INDEX)
                    .try_send(message.to_packet(), peer)
                    .is_err()
                {
                    error!(
                        "failed to send reliable packet to {peer}: {message:?}"
                    );
                }
            });
        }
        if !queue.reliable_to_all_except.is_empty() {
            trace!(
                "sent {} [R;N-1] {} packets",
                queue.reliable_to_all_except.len(),
                M::reflect_name()
            );
        }
        for (peer, message) in queue.unreliable_to_all_except.iter() {
            let peers: Vec<PeerId> =
                socket.connected_peers().filter(|p| p != peer).collect();
            peers.into_iter().for_each(|peer| {
                    if socket
                        .channel_mut(UNRELIABLE_CHANNEL_INDEX)
                        .try_send(message.to_packet(), peer).is_err() {
                        error!("failed to send unreliable packet to {peer}: {message:?}");
                    }
                });
        }
        if !queue.unreliable_to_all_except.is_empty() {
            trace!(
                "sent {} [U;N-1] {} packets",
                queue.unreliable_to_all_except.len(),
                M::reflect_name()
            );
        }
        for (peer, message) in queue.reliable_to_peer.iter() {
            if socket
                .channel_mut(RELIABLE_CHANNEL_INDEX)
                .try_send(message.to_packet(), *peer)
                .is_err()
            {
                error!("failed to send reliable packet to {peer}: {message:?}");
            }
        }
        if !queue.reliable_to_peer.is_empty() {
            trace!(
                "sent {} [R] {} packets",
                queue.reliable_to_peer.len(),
                M::reflect_name()
            );
        }
        for (peer, message) in queue.unreliable_to_peer.iter() {
            if socket
                .channel_mut(UNRELIABLE_CHANNEL_INDEX)
                .try_send(message.to_packet(), *peer)
                .is_err()
            {
                error!(
                    "failed to send unreliable packet to {peer}: {message:?}"
                );
            }
        }
        if !queue.unreliable_to_peer.is_empty() {
            trace!(
                "sent {} [U] {} packets",
                queue.unreliable_to_peer.len(),
                M::reflect_name()
            );
        }

        queue.flush();
    }
}
