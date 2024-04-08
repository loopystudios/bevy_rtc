use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Payload;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam, Debug)]
pub struct RtcClient<'w, M: Payload> {
    pub(crate) incoming: ResMut<'w, IncomingMessages<M>>,
    pub(crate) outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Payload> RtcClient<'w, M> {
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

    /// Iterate over all messages in the buffer without consuming them.
    pub fn iter(&mut self) -> impl Iterator<Item = &M> {
        self.incoming.messages.iter()
    }

    /// Clear all messages waiting in the buffer.
    pub fn clear(&mut self) {
        self.incoming.messages.clear()
    }

    /// Consumes all messages in the buffer and iterate on them.
    pub fn read(&mut self) -> std::collections::vec_deque::Drain<'_, M> {
        self.incoming.messages.drain(..)
    }

    /// Send a payload to the host with reliability. The payload is created with
    /// lazy behavior, only when the send rate allows.
    pub fn reliable_to_host_with(&mut self, message_fn: impl Fn() -> M) {
        self.outgoing.reliable_to_host.push(message_fn());
    }

    /// Send a payload to the host with no expectation of delivery. The payload
    /// is created with lazy behavior, only when the send rate allows.
    pub fn unreliable_to_host_with(&mut self, message_fn: impl Fn() -> M) {
        self.outgoing.unreliable_to_host.push(message_fn());
    }

    /// Send a payload to the host with reliability.
    pub fn reliable_to_host(&mut self, message: M) {
        self.outgoing.reliable_to_host.push(message);
    }

    /// Send a payload to the host with no expectation of delivery.
    pub fn unreliable_to_host(&mut self, message: M) {
        self.outgoing.unreliable_to_host.push(message);
    }
}
