#[cfg(all(target_arch = "wasm32", feature = "server"))]
compile_error!(
    "The 'server' feature is not supported on the wasm32 target architecture."
);

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

pub use silk_common as common;
pub use silk_net as net;
