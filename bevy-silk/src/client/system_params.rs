use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Payload;
use bevy::{
    ecs::system::{SystemChangeTick, SystemParam},
    prelude::*,
};
use instant::Duration;

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Payload> {
    incoming: ResMut<'w, IncomingMessages<M>>,
}

impl<'w, M: Payload> NetworkReader<'w, M> {
    /// Consumes all messages in the buffer and iterate on them.
    pub fn drain(&mut self) -> std::vec::Drain<M> {
        self.incoming.messages.drain(..)
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, 's, M: Payload, const RATE_MS: u64 = 0> {
    pub(crate) tick: SystemChangeTick,
    pub(crate) timer: Local<'s, Option<Timer>>,
    pub(crate) last_tick: Local<'s, u32>,
    pub(crate) last_instant: Local<'s, Option<instant::Instant>>,
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
                let now = instant::Instant::now();
                let last_instant = self.last_instant.get_or_insert(now);
                timer.tick(now - *last_instant);
                *last_instant = now;
                *self.last_tick = tick;
            }
            timer.finished()
        }
    }

    /// Send a payload to the host with reliability. The payload is created with
    /// lazy behavior, only when the send rate allows.
    pub fn reliable_to_host_with(&mut self, message_fn: impl Fn() -> M) {
        if self.ready() {
            self.outgoing.reliable_to_host.push(message_fn());
        }
    }

    /// Send a payload to the host with no expectation of delivery. The payload
    /// is created with lazy behavior, only when the send rate allows.
    pub fn unreliable_to_host_with(&mut self, message_fn: impl Fn() -> M) {
        if self.ready() {
            self.outgoing.unreliable_to_host.push(message_fn());
        }
    }
}

impl<'w, 's, M: Payload> NetworkWriter<'w, 's, M, 0> {
    /// Send a payload to the host with reliability.
    pub fn reliable_to_host(&mut self, message: M) {
        if self.ready() {
            self.outgoing.reliable_to_host.push(message);
        }
    }

    /// Send a payload to the host with no expectation of delivery.
    pub fn unreliable_to_host(&mut self, message: M) {
        if self.ready() {
            self.outgoing.unreliable_to_host.push(message);
        }
    }
}
