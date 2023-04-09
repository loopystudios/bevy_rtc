use bevy::{
    ecs::schedule::{ScheduleLabel, SystemSetConfigs},
    prelude::*,
};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SilkServerSchedule;

pub fn run_silk_schedule(world: &mut World) {
    world.run_schedule(SilkServerSchedule);
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
#[system_set(base)]
pub enum SilkServerStage {
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

impl SilkServerStage {
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
