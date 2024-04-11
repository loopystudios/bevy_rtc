#[allow(clippy::module_inception)]
mod client;
mod events;
mod plugin;
mod router;
mod state;
mod systems;

pub use client::RtcClient;
pub use events::{RtcClientEvent, RtcClientRequestEvent};
pub use plugin::RtcClientPlugin;
pub use router::AddClientProtocolExt;
pub use state::{RtcClientState, RtcClientStatus};
