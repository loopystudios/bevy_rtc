use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Protocol;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_matchbox::prelude::PeerId;

/// A [`SystemParam`] for reading payloads of a particular type.
#[derive(SystemParam, Debug)]
pub struct RtcServer<'w, M: Protocol> {
    // Option is none if it's send-only or read-only.
    pub(crate) incoming: Option<ResMut<'w, IncomingMessages<M>>>,
    pub(crate) outgoing: Option<ResMut<'w, OutgoingMessages<M>>>,
}

impl<'w, M: Protocol> RtcServer<'w, M> {
    /// Returns the capacity of incoming messages.
    pub fn capacity(&self) -> usize {
        self.incoming.as_ref().map(|v| v.bound).unwrap_or(0)
    }

    /// Returns the number of messages waiting in the buffer without draining them.
    pub fn len(&self) -> usize {
        self.incoming
            .as_ref()
            .map(|v| v.messages.len())
            .unwrap_or(0)
    }

    /// Returns the number of messages waiting in the buffer without draining them.
    pub fn is_empty(&self) -> bool {
        self.incoming
            .as_ref()
            .map(|v| v.messages.is_empty())
            .unwrap_or(true)
    }

    /// Clear all messages waiting in the buffer.
    pub fn clear(&mut self) {
        if let Some(ref mut incoming) = self.incoming {
            incoming.messages.clear()
        }
    }

    /// Consumes all messages in the buffer and iterate on them.
    pub fn read(&mut self) -> Vec<(PeerId, M)> {
        if let Some(ref mut incoming) = self.incoming {
            incoming
                .messages
                .drain()
                .fold(vec![], |mut v, (peer, payloads)| {
                    v.extend(payloads.into_iter().map(|p| (peer, p)));
                    v
                })
        } else {
            panic!(
                "Attempting to read from `{}` is not allowed, it is registered write only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers with reliability.
    pub fn reliable_to_all(&mut self, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_all.push(message);
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers with no expectation of delivery.
    pub fn unreliable_to_all(&mut self, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_all.push(message);
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to a peer with reliability.
    pub fn reliable_to_peer(&mut self, peer_id: PeerId, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_peer.push((peer_id, message));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to a peer with no expectation of delivery.
    pub fn unreliable_to_peer(&mut self, peer_id: PeerId, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.unreliable_to_peer.push((peer_id, message));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers except one with reliability.
    pub fn reliable_to_all_except(&mut self, peer_id: PeerId, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_all_except.push((peer_id, message));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers except one with no expectation of
    /// delivery.
    pub fn unreliable_to_all_except(&mut self, peer_id: PeerId, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.unreliable_to_all_except.push((peer_id, message));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers with reliability. The payload is
    /// created with lazy behavior, only when the send rate allows.
    pub fn reliable_to_all_with(&mut self, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_all.push(message_fn());
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers with no expectation of delivery.
    /// The payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn unreliable_to_all_with(&mut self, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.unreliable_to_all.push(message_fn());
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to a peer with reliability. The payload is
    /// created with lazy behavior, only when the send rate allows.
    pub fn reliable_to_peer_with(&mut self, peer_id: PeerId, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_peer.push((peer_id, message_fn()));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to a peer with no expectation of delivery.
    /// The payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn unreliable_to_peer_with(&mut self, peer_id: PeerId, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.unreliable_to_peer.push((peer_id, message_fn()));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers except one with reliability. The
    /// payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn reliable_to_all_except_with(&mut self, peer_id: PeerId, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing
                .reliable_to_all_except
                .push((peer_id, message_fn()));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to all connected peers except one with no expectation of
    /// delivery. The payload is created with lazy behavior, only when the
    /// send rate allows.
    pub fn unreliable_to_all_except_with(&mut self, peer_id: PeerId, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing
                .unreliable_to_all_except
                .push((peer_id, message_fn()));
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }
}
