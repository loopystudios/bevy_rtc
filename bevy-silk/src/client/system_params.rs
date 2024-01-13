use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Payload;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Payload> {
    incoming: ResMut<'w, IncomingMessages<M>>,
}

impl<'w, M: Payload> NetworkReader<'w, M> {
    /// Consumes all messages in the buffer and iterate on them.
    pub fn read(&mut self) -> std::collections::vec_deque::Drain<'_, M> {
        self.incoming.messages.drain(..)
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, M: Payload> {
    pub(crate) outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Payload> NetworkWriter<'w, M> {
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
