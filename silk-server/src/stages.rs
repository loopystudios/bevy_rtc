use bevy::{
    ecs::schedule::{ScheduleLabel, SystemSetConfigs},
    prelude::*,
};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SilkStagesSchedule;

pub fn run_silk_schedule(world: &mut World) {
    error!("FIXED");
    world.run_schedule(SilkStagesSchedule);
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
#[system_set(base)]
pub enum SilkStages {
    /// Silk plugin reads from silk socket and sends "incoming client
    /// message" events
    ReadSocket,
    /// Game receives "incoming client message" events from Silk plugin
    /// and creates "side effects"
    ProcessIncomingEvents,
    /// Game updates world state here with the "side effects"
    UpdateWorldState,
    /// Game sends broadcast events to Silk plugin (after world state
    /// reacts with "side effects" to create a new world state)
    ProcessOutgoingEvents,
    /// Silk plugin reads broadcast events game and sends messages over
    /// the silk socket
    WriteSocket,
}

impl SilkStages {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::ReadSocket,
            Self::ProcessIncomingEvents,
            Self::UpdateWorldState,
            Self::ProcessOutgoingEvents,
            Self::WriteSocket,
        ]
        .into_iter()
    }

    pub fn sets() -> SystemSetConfigs {
        (
            Self::ReadSocket,
            Self::ProcessIncomingEvents,
            Self::UpdateWorldState,
            Self::ProcessOutgoingEvents,
            Self::WriteSocket,
        )
            .chain()
    }
}
