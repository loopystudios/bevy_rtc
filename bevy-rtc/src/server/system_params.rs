use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Payload;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_matchbox::prelude::PeerId;

/// A [`SystemParam`] for reading payloads of a particular type.
#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Payload> {
    incoming: ResMut<'w, IncomingMessages<M>>,
}

impl<'w, M: Payload> NetworkReader<'w, M> {
    /// Returns the capacity of this network reader.
    pub fn capacity(&self) -> usize {
        self.incoming.bound
    }

    /// Returns the number of messages waiting in the buffer without draining them.
    pub fn len(&self) -> usize {
        self.incoming.messages.len()
    }

    /// Returns the number of messages waiting in the buffer without draining them.
    pub fn is_empty(&self) -> bool {
        self.incoming.messages.is_empty()
    }

    /// Consumes all messages in the buffer and iterate on them.
    pub fn read(&mut self) -> Vec<(PeerId, M)> {
        self.incoming.messages.drain().fold(
            vec![],
            |mut v, (peer, payloads)| {
                v.extend(payloads.into_iter().map(|p| (peer, p)));
                v
            },
        )
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, M: Payload> {
    pub(crate) outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Payload> NetworkWriter<'w, M> {
    /// Send a payload to all connected peers with reliability.
    pub fn reliable_to_all(&mut self, message: M) {
        self.outgoing.reliable_to_all.push(message);
    }

    /// Send a payload to all connected peers with no expectation of delivery.
    pub fn unreliable_to_all(&mut self, message: M) {
        self.outgoing.unreliable_to_all.push(message);
    }

    /// Send a payload to a peer with reliability.
    pub fn reliable_to_peer(&mut self, peer_id: PeerId, message: M) {
        self.outgoing.reliable_to_peer.push((peer_id, message));
    }

    /// Send a payload to a peer with no expectation of delivery.
    pub fn unreliable_to_peer(&mut self, peer_id: PeerId, message: M) {
        self.outgoing.unreliable_to_peer.push((peer_id, message));
    }

    /// Send a payload to all connected peers except one with reliability.
    pub fn reliable_to_all_except(&mut self, peer_id: PeerId, message: M) {
        self.outgoing
            .reliable_to_all_except
            .push((peer_id, message));
    }

    /// Send a payload to all connected peers except one with no expectation of
    /// delivery.
    pub fn unreliable_to_all_except(&mut self, peer_id: PeerId, message: M) {
        self.outgoing
            .unreliable_to_all_except
            .push((peer_id, message));
    }

    /// Send a payload to all connected peers with reliability. The payload is
    /// created with lazy behavior, only when the send rate allows.
    pub fn reliable_to_all_with(&mut self, message_fn: impl Fn() -> M) {
        self.outgoing.reliable_to_all.push(message_fn());
    }

    /// Send a payload to all connected peers with no expectation of delivery.
    /// The payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn unreliable_to_all_with(&mut self, message_fn: impl Fn() -> M) {
        self.outgoing.unreliable_to_all.push(message_fn());
    }

    /// Send a payload to a peer with reliability. The payload is
    /// created with lazy behavior, only when the send rate allows.
    pub fn reliable_to_peer_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        self.outgoing.reliable_to_peer.push((peer_id, message_fn()));
    }

    /// Send a payload to a peer with no expectation of delivery.
    /// The payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn unreliable_to_peer_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        self.outgoing
            .unreliable_to_peer
            .push((peer_id, message_fn()));
    }

    /// Send a payload to all connected peers except one with reliability. The
    /// payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn reliable_to_all_except_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        self.outgoing
            .reliable_to_all_except
            .push((peer_id, message_fn()));
    }

    /// Send a payload to all connected peers except one with no expectation of
    /// delivery. The payload is created with lazy behavior, only when the
    /// send rate allows.
    pub fn unreliable_to_all_except_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        self.outgoing
            .unreliable_to_all_except
            .push((peer_id, message_fn()));
    }
}
