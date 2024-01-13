use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Payload;
use bevy::{
    ecs::system::{SystemChangeTick, SystemParam},
    prelude::*,
};
use bevy_matchbox::prelude::PeerId;
use instant::Duration;

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, 's, M: Payload, const RATE_MS: u64 = 0> {
    pub(crate) tick: SystemChangeTick,
    pub(crate) time: Res<'w, Time>,
    pub(crate) timer: Local<'s, Option<Timer>>,
    pub(crate) last_tick: Local<'s, u32>,
    pub(crate) incoming: ResMut<'w, IncomingMessages<M>>,
}

impl<'w, 's, M: Payload, const RATE_MS: u64> NetworkReader<'w, 's, M, RATE_MS> {
    /// Returns true if the time since the last tick has passed the rate
    /// duration.
    #[inline]
    pub(crate) fn ready(&mut self) -> bool {
        if RATE_MS == 0 {
            true
        } else {
            let tick = self.tick.this_run().get();
            let timer = self.timer.get_or_insert(Timer::new(
                Duration::from_millis(RATE_MS),
                TimerMode::Repeating,
            ));
            if *self.last_tick != tick {
                timer.tick(self.time.delta());
                *self.last_tick = tick;
            }
            timer.finished()
        }
    }

    /// Consumes all messages in the buffer and iterate on them.
    pub fn read(&mut self) -> std::vec::Drain<(PeerId, M)> {
        if self.ready() {
            self.incoming.messages.drain(..)
        } else {
            self.incoming.messages.drain(0..0)
        }
    }

    /// Iterate over the messages in the buffer, without consuming them.
    /// This is useful if you have two systems which both need to read the same
    /// payload for different reasons.
    pub fn iter(&mut self) -> core::slice::Iter<'_, (PeerId, M)> {
        if self.ready() {
            self.incoming.messages.iter()
        } else {
            [].iter()
        }
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, 's, M: Payload, const RATE_MS: u64 = 0> {
    pub(crate) tick: SystemChangeTick,
    pub(crate) time: Res<'w, Time>,
    pub(crate) timer: Local<'s, Option<Timer>>,
    pub(crate) last_tick: Local<'s, u32>,
    pub(crate) outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, 's, M: Payload, const RATE_MS: u64> NetworkWriter<'w, 's, M, RATE_MS> {
    /// Returns true if the time since the last tick has passed the rate
    /// duration.
    #[inline]
    pub(crate) fn ready(&mut self) -> bool {
        if RATE_MS == 0 {
            true
        } else {
            let tick = self.tick.this_run().get();
            let timer = self.timer.get_or_insert(Timer::new(
                Duration::from_millis(RATE_MS),
                TimerMode::Repeating,
            ));
            if *self.last_tick != tick {
                timer.tick(self.time.delta());
                *self.last_tick = tick;
            }
            timer.finished()
        }
    }

    /// Send a payload to all connected peers with reliability. The payload is
    /// only sent when the send rate allows.
    pub fn reliable_to_all(&mut self, message: M) {
        if self.ready() {
            self.outgoing.reliable_to_all.push(message);
        }
    }

    /// Send a payload to all connected peers with no expectation of delivery.
    /// The payload is only sent when the send rate allows.
    pub fn unreliable_to_all(&mut self, message: M) {
        if self.ready() {
            self.outgoing.unreliable_to_all.push(message);
        }
    }

    /// Send a payload to all connected peers with reliability. The payload is
    /// created with lazy behavior, only when the send rate allows.
    pub fn reliable_to_all_with(&mut self, message_fn: impl Fn() -> M) {
        if self.ready() {
            self.outgoing.reliable_to_all.push(message_fn());
        }
    }

    /// Send a payload to all connected peers with no expectation of delivery.
    /// The payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn unreliable_to_all_with(&mut self, message_fn: impl Fn() -> M) {
        if self.ready() {
            self.outgoing.unreliable_to_all.push(message_fn());
        }
    }

    /// Send a payload to a peer with reliability. The payload is
    /// only sent when the send rate allows.
    pub fn reliable_to_peer(&mut self, peer_id: PeerId, message: M) {
        if self.ready() {
            self.outgoing.reliable_to_peer.push((peer_id, message));
        }
    }

    /// Send a payload to a peer with no expectation of delivery.
    /// The payload is only sent when the send rate allows.
    pub fn unreliable_to_peer(&mut self, peer_id: PeerId, message: M) {
        if self.ready() {
            self.outgoing.unreliable_to_peer.push((peer_id, message));
        }
    }

    /// Send a payload to a peer with reliability. The payload is
    /// created with lazy behavior, only when the send rate allows.
    pub fn reliable_to_peer_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        if self.ready() {
            self.outgoing.reliable_to_peer.push((peer_id, message_fn()));
        }
    }

    /// Send a payload to a peer with no expectation of delivery.
    /// The payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn unreliable_to_peer_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        if self.ready() {
            self.outgoing
                .unreliable_to_peer
                .push((peer_id, message_fn()));
        }
    }

    /// Send a payload to all connected peers except one with reliability. The
    /// payload is only sent when the send rate allows.
    pub fn reliable_to_all_except(&mut self, peer_id: PeerId, message: M) {
        if self.ready() {
            self.outgoing
                .reliable_to_all_except
                .push((peer_id, message));
        }
    }

    /// Send a payload to all connected peers except one with no expectation of
    /// delivery. The payload is only sent when the send rate allows.
    pub fn unreliable_to_all_except(&mut self, peer_id: PeerId, message: M) {
        if self.ready() {
            self.outgoing
                .unreliable_to_all_except
                .push((peer_id, message));
        }
    }

    /// Send a payload to all connected peers except one with reliability. The
    /// payload is created with lazy behavior, only when the send rate
    /// allows.
    pub fn reliable_to_all_except_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        if self.ready() {
            self.outgoing
                .reliable_to_all_except
                .push((peer_id, message_fn()));
        }
    }

    /// Send a payload to all connected peers except one with no expectation of
    /// delivery. The payload is created with lazy behavior, only when the
    /// send rate allows.
    pub fn unreliable_to_all_except_with(
        &mut self,
        peer_id: PeerId,
        message_fn: impl Fn() -> M,
    ) {
        if self.ready() {
            self.outgoing
                .unreliable_to_all_except
                .push((peer_id, message_fn()));
        }
    }
}
