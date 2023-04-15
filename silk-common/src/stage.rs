use bevy::{ecs::schedule::SystemSetConfigs, prelude::*};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
#[system_set(base)]
pub enum SilkStage {
    /// An exclusive system to read Silk events.
    ReadIn,
    /// An exclusive system to process latency.
    ProcessLatency,
    /// Default stage for game updates.
    Update,
    /// The last opportunity to write Silk broadcast events this tick.
    WriteOut,
}

impl SilkStage {
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
