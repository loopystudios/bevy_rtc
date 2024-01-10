#[cfg(all(target_arch = "wasm32", feature = "server"))]
compile_error!(
    "The 'server' feature is not supported on the wasm32 target architecture."
);

pub(crate) mod events;
pub(crate) mod latency;
pub mod protocol;
pub(crate) mod socket;

// Re-exports
pub use bevy_matchbox;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;
