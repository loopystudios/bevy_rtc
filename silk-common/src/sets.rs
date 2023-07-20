use bevy::{ecs::schedule::SystemSetConfigs, prelude::*};
use strum_macros::{Display, EnumIter};

#[derive(
    Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet, Display, EnumIter,
)]
pub enum SilkSet {
    /// Do not use this system, it flushes previous network buffers since we do not consume on read for network traffic.
    Flush,
    /// An exclusive system to read network traffic
    NetworkRead,
    /// A system to process network traffic.
    Process,
    /// An exclusive system to receive Silk events
    SilkEvents,
    /// Apply updates before the main update
    PreUpdate,
    /// Default stage for game updates.
    Update,
    /// Apply updates after the main update
    PostUpdate,
    /// The last opportunity to write network traffic
    NetworkWrite,
}

impl SilkSet {
    pub fn sets() -> SystemSetConfigs {
        // Define the ordering of systems here
        (
            Self::Flush,
            Self::NetworkRead,
            Self::Process,
            Self::SilkEvents,
            Self::PreUpdate,
            Self::Update,
            Self::PostUpdate,
            Self::NetworkWrite,
        )
            .chain()
    }
}
