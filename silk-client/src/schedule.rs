use bevy::{
    ecs::schedule::{ScheduleLabel, SystemSetConfigs},
    prelude::*,
};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SilkClientSchedule;

pub fn run_silk_schedule(world: &mut World) {
    world.run_schedule(SilkClientSchedule);
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
#[system_set(base)]
pub enum SilkClientStage {
    /// Always the first stage, receiving network events from the silk socket
    ReadSocket,
    /// Game updates world state here with the "side effects"
    UpdateWorldState,
    /// Always the last stage, forward all client messages to the server
    WriteSocket,
}

impl SilkClientStage {
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::ReadSocket, Self::UpdateWorldState, Self::WriteSocket]
            .into_iter()
    }

    pub fn sets() -> SystemSetConfigs {
        (Self::ReadSocket, Self::UpdateWorldState, Self::WriteSocket).chain()
    }
}
