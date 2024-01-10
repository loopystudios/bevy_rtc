#[cfg(all(target_arch = "wasm32", feature = "server"))]
compile_error!(
    "The 'server' feature is not supported on the wasm32 target architecture."
);

#[cfg(not(any(feature = "server", feature = "client")))]
compile_error!("Either 'server' or 'client' feature must be enabled.");

#[cfg(all(feature = "server", feature = "client"))]
compile_error!(
    "Cannot enable both 'server' and 'client' features simultaneously."
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
