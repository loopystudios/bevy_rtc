use bevy::{ecs::schedule::SystemSetConfigs, prelude::*};
use strum_macros::{Display, EnumIter};

#[derive(
    Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet, Display, EnumIter,
)]
#[system_set(base)]
pub enum SilkStage {
    /// An exclusive system to read network traffic
    NetworkRead,
    /// A system to process network traffic.
    Process,
    /// An exclusive system to receive Silk events
    SilkEvents,
    /// Default stage for game updates.
    Update,
    /// The last opportunity to write network traffic
    NetworkWrite,
}

impl SilkStage {
    pub fn sets() -> SystemSetConfigs {
        // Define the ordering of systems here
        (
            Self::NetworkRead,
            Self::Process,
            Self::SilkEvents,
            Self::Update,
            Self::NetworkWrite,
        )
            .chain()
    }
}
