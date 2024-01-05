#[cfg(all(target_arch = "wasm32", feature = "server"))]
compile_error!(
    "The 'server' feature is not supported on the wasm32 target architecture."
);

pub(crate) mod common_plugin;
pub mod events;
pub mod packets;
pub mod protocol;
pub mod schedule;
pub mod sets;
pub mod socket;

// Re-exports
pub use bevy_matchbox;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;
