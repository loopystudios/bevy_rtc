use super::router::{IncomingMessages, OutgoingMessages};
use crate::protocol::Payload;
use bevy::{
    ecs::{
        component::Tick,
        system::{
            ExclusiveSystemParam, ReadOnlySystemParam, SystemMeta, SystemParam,
        },
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    prelude::*,
    utils::synccell::SyncCell,
};
use bevy_matchbox::prelude::PeerId;
use instant::Duration;

#[derive(Debug)]
pub struct NetworkReader<'w, M: Payload, const RATE_MS: u64 = 0> {
    pub(crate) timer: &'w mut Timer,
    pub(crate) incoming: &'w mut IncomingMessages<M>,
}

impl<'w, M: Payload, const RATE_MS: u64> NetworkReader<'w, M, RATE_MS> {
    /// Returns true if the time since the last tick has passed the rate
    /// duration.
    #[inline]
    pub(crate) fn ready(&self) -> bool {
        if RATE_MS == 0 {
            true
        } else {
            self.timer.finished()
        }
    }

    /// Consumes all messages in the buffer and iterate on them.
    pub fn read(&mut self) -> std::vec::Drain<(PeerId, M)> {
        if self.ready() {
            error!("{}", self.incoming.messages.len());
            self.incoming.messages.drain(..)
        } else {
            error!("{}", self.incoming.messages.len());
            self.incoming.messages.drain(0..0)
        }
    }

    /// Iterate over the messages in the buffer, without consuming them.
    /// This is useful if you have two systems which both need to read the same
    /// payload for different reasons.
    pub fn iter(&self) -> core::slice::Iter<'_, (PeerId, M)> {
        if self.ready() {
            self.incoming.messages.iter()
        } else {
            [].iter()
        }
    }
}

impl<'_s, M: Payload, const RATE_MS: u64> ExclusiveSystemParam
    for NetworkReader<'_s, M, RATE_MS>
{
    type State = SyncCell<(Timer, instant::Instant, IncomingMessages<M>)>;
    type Item<'s> = NetworkReader<'s, M, RATE_MS>;

    fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        let timer =
            Timer::new(Duration::from_millis(RATE_MS), TimerMode::Repeating);
        let instant = instant::Instant::now();
        let incoming = IncomingMessages::<M> { messages: vec![] };
        SyncCell::new((timer, instant, incoming))
    }

    fn get_param<'s>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
    ) -> Self::Item<'s> {
        let now = instant::Instant::now();
        let (timer, last_instant, incoming) = state.get();
        timer.tick(now - *last_instant);
        *last_instant = now;
        NetworkReader { timer, incoming }
    }
}

// SAFETY: only local state is accessed
unsafe impl<'s, M: Payload, const RATE_MS: u64> ReadOnlySystemParam
    for NetworkReader<'s, M, RATE_MS>
{
}

// SAFETY: only local state is accessed
unsafe impl<'a, M: Payload, const RATE_MS: u64> SystemParam
    for NetworkReader<'a, M, RATE_MS>
{
    type State = SyncCell<(Timer, instant::Instant, IncomingMessages<M>)>;
    type Item<'w, 's> = NetworkReader<'s, M, RATE_MS>;

    fn init_state(
        _world: &mut World,
        _system_meta: &mut SystemMeta,
    ) -> Self::State {
        let timer =
            Timer::new(Duration::from_millis(RATE_MS), TimerMode::Repeating);
        let instant = instant::Instant::now();
        let incoming = IncomingMessages::<M> { messages: vec![] };
        SyncCell::new((timer, instant, incoming))
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
        _world: UnsafeWorldCell<'w>,
        _change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        let now = instant::Instant::now();
        let (timer, last_instant, incoming) = state.get();
        timer.tick(now - *last_instant);
        *last_instant = now;
        NetworkReader { timer, incoming }
    }
}

#[derive(Debug)]
pub struct NetworkWriter<'w, M: Payload, const RATE_MS: u64 = 0> {
    pub(crate) timer: &'w mut Timer,
    pub(crate) outgoing: &'w mut OutgoingMessages<M>,
}

impl<'w, M: Payload, const RATE_MS: u64> NetworkWriter<'w, M, RATE_MS> {
    /// Returns true if the time since the last tick has passed the rate
    /// duration.
    #[inline]
    pub(crate) fn ready(&self) -> bool {
        if RATE_MS == 0 {
            true
        } else {
            self.timer.finished()
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

impl<'_s, M: Payload, const RATE_MS: u64> ExclusiveSystemParam
    for NetworkWriter<'_s, M, RATE_MS>
{
    type State = SyncCell<(Timer, instant::Instant, OutgoingMessages<M>)>;
    type Item<'s> = NetworkWriter<'s, M, RATE_MS>;

    fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        let timer =
            Timer::new(Duration::from_millis(RATE_MS), TimerMode::Repeating);
        let instant = instant::Instant::now();
        let outgoing = OutgoingMessages::<M> {
            reliable_to_peer: vec![],
            reliable_to_all: vec![],
            reliable_to_all_except: vec![],
            unreliable_to_peer: vec![],
            unreliable_to_all: vec![],
            unreliable_to_all_except: vec![],
        };
        SyncCell::new((timer, instant, outgoing))
    }

    fn get_param<'s>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
    ) -> Self::Item<'s> {
        let now = instant::Instant::now();
        let (timer, last_instant, outgoing) = state.get();
        timer.tick(now - *last_instant);
        *last_instant = now;
        NetworkWriter { timer, outgoing }
    }
}

// SAFETY: only local state is accessed
unsafe impl<'s, M: Payload, const RATE_MS: u64> ReadOnlySystemParam
    for NetworkWriter<'s, M, RATE_MS>
{
}

// SAFETY: only local state is accessed
unsafe impl<'a, M: Payload, const RATE_MS: u64> SystemParam
    for NetworkWriter<'a, M, RATE_MS>
{
    type State = SyncCell<(Timer, instant::Instant, OutgoingMessages<M>)>;
    type Item<'w, 's> = NetworkWriter<'s, M, RATE_MS>;

    fn init_state(
        _world: &mut World,
        _system_meta: &mut SystemMeta,
    ) -> Self::State {
        let timer =
            Timer::new(Duration::from_millis(RATE_MS), TimerMode::Repeating);
        let instant = instant::Instant::now();
        let outgoing = OutgoingMessages::<M> {
            reliable_to_peer: vec![],
            reliable_to_all: vec![],
            reliable_to_all_except: vec![],
            unreliable_to_peer: vec![],
            unreliable_to_all: vec![],
            unreliable_to_all_except: vec![],
        };
        SyncCell::new((timer, instant, outgoing))
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
        _world: UnsafeWorldCell<'w>,
        _change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        let now = instant::Instant::now();
        let (timer, last_instant, outgoing) = state.get();
        timer.tick(now - *last_instant);
        *last_instant = now;
        NetworkWriter { timer, outgoing }
    }
}
