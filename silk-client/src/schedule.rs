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
    /// An exclusive system to read Silk events.
    ReadIn,
    /// An exclusive system to process latency.
    ProcessLatency,
    /// Default stage for game updates.
    Update,
    /// The last opportunity to write Silk broadcast events this tick.
    WriteOut,
}

impl SilkClientStage {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::ReadIn,
            Self::ProcessLatency,
            Self::Update,
            Self::WriteOut,
        ]
        .into_iter()
    }

    pub fn sets() -> SystemSetConfigs {
        (
            Self::ReadIn,
            Self::ProcessLatency,
            Self::Update,
            Self::WriteOut,
        )
            .chain()
    }
}
