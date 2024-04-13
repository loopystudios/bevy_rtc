#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(target_arch = "wasm32", feature = "server"))]
compile_error!("The 'server' feature is not supported on the wasm32 target architecture.");

pub(crate) mod events;
pub(crate) mod latency;
pub mod protocol;
pub(crate) mod socket;

mod transport_encoding;
pub use transport_encoding::TransportEncoding;

// Re-exports
pub use bevy_matchbox;

pub mod prelude {
    #[cfg(feature = "client")]
    pub use crate::client::*;
    #[cfg(feature = "server")]
    pub use crate::server::*;

    pub use crate::{protocol::Protocol, TransportEncoding};
}

#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
pub mod server;

#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub mod client;
