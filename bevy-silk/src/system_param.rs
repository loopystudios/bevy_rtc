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
use instant::Duration;

/// A Bevy [`SystemParam`] to make throttling send and receive events easy.
pub struct NetworkThrottle<'s, const MS: u64> {
    pub(crate) timer: &'s mut Timer,
}

impl<const MS: u64> NetworkThrottle<'_, MS> {
    /// Returns true if the time since the last tick has passed the throttle
    /// duration.
    pub fn ready(&self) -> bool {
        self.timer.just_finished()
    }
}

impl<'_s, const MS: u64> ExclusiveSystemParam for NetworkThrottle<'_s, MS> {
    type State = SyncCell<(Timer, instant::Instant)>;
    type Item<'s> = NetworkThrottle<'s, MS>;

    fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        let timer = Timer::new(Duration::from_millis(MS), TimerMode::Repeating);
        let instant = instant::Instant::now();
        SyncCell::new((timer, instant))
    }

    fn get_param<'s>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
    ) -> Self::Item<'s> {
        let now = instant::Instant::now();
        let (timer, last_instant) = state.get();
        timer.tick(now - *last_instant);
        *last_instant = now;
        NetworkThrottle { timer }
    }
}

// SAFETY: only local state is accessed
unsafe impl<'s, const MS: u64> ReadOnlySystemParam for NetworkThrottle<'s, MS> {}

// SAFETY: only local state is accessed
unsafe impl<'a, const MS: u64> SystemParam for NetworkThrottle<'a, MS> {
    type State = SyncCell<(Timer, instant::Instant)>;
    type Item<'w, 's> = NetworkThrottle<'s, MS>;

    fn init_state(
        _world: &mut World,
        _system_meta: &mut SystemMeta,
    ) -> Self::State {
        let timer = Timer::new(Duration::from_millis(MS), TimerMode::Repeating);
        let instant = instant::Instant::now();
        SyncCell::new((timer, instant))
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
        _world: UnsafeWorldCell<'w>,
        _change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        let now = instant::Instant::now();
        let (timer, last_instant) = state.get();
        timer.tick(now - *last_instant);
        *last_instant = now;
        NetworkThrottle { timer }
    }
}
