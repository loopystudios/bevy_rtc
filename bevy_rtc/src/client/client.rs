use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Protocol;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam, Debug)]
pub struct RtcClient<'w, M: Protocol> {
    // Option is none if it's send-only or read-only.
    pub(crate) incoming: Option<ResMut<'w, IncomingMessages<M>>>,
    pub(crate) outgoing: Option<ResMut<'w, OutgoingMessages<M>>>,
}

impl<'w, M: Protocol> RtcClient<'w, M> {
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
    pub fn read(&mut self) -> Vec<M> {
        if let Some(ref mut incoming) = self.incoming {
            incoming.messages.drain(..).collect()
        } else {
            panic!(
                "Attempting to read from `{}` is not allowed, it is registered write only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to the host with reliability. The payload is created with
    /// lazy behavior, only when the send rate allows.
    pub fn reliable_to_host_with(&mut self, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_host.push(message_fn());
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to the host with no expectation of delivery. The payload
    /// is created with lazy behavior, only when the send rate allows.
    pub fn unreliable_to_host_with(&mut self, message_fn: impl Fn() -> M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.unreliable_to_host.push(message_fn());
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to the host with reliability.
    pub fn reliable_to_host(&mut self, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.reliable_to_host.push(message);
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }

    /// Send a payload to the host with no expectation of delivery.
    pub fn unreliable_to_host(&mut self, message: M) {
        if let Some(ref mut outgoing) = self.outgoing {
            outgoing.unreliable_to_host.push(message);
        } else {
            panic!(
                "Attempting to write `{}` is not allowed, it is registered read only.",
                M::reflect_name()
            );
        }
    }
}
