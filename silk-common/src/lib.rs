use bevy_matchbox::matchbox_socket::{
    ChannelConfig, MessageLoopFuture, MultipleChannels, WebRtcSocket,
    WebRtcSocketBuilder,
};
use std::net::IpAddr;

pub mod demo_packets;
pub mod network_queries;
pub mod packets;
pub mod stage;

// Re-exports
pub use bevy_matchbox;
pub use network_queries::AddNetworkQuery;
pub use stage::SilkStage;

/// An abstraction over [`matchbox_socket::WebRtcSocket`] to fit Tribrid's
/// Client-Server architecture.
pub struct SilkSocket {
    builder: WebRtcSocketBuilder<MultipleChannels>,
}

impl SilkSocket {
    /// The index of the unreliable channel in the [`WebRtcSocket`].
    pub const UNRELIABLE_CHANNEL_INDEX: usize = 0;
    /// The index of the reliable channel in the [`WebRtcSocket`].
    pub const RELIABLE_CHANNEL_INDEX: usize = 1;

    /// Initialize a WebRTC socket compatible with Tribrid's Client-Server
    /// architecture.
    pub fn new(config: ConnectionAddr) -> Self {
        let room_url = config.to_url();
        let builder = WebRtcSocket::builder(room_url)
            .add_channel(ChannelConfig {
                ordered: true,
                max_retransmits: Some(0),
            }) // Match UNRELIABLE_CHANNEL_INDEX
            .add_channel(ChannelConfig::reliable()); // Match RELIABLE_CHANNEL_INDEX

        Self { builder }
    }

    pub fn builder(self) -> WebRtcSocketBuilder<MultipleChannels> {
        self.builder
    }

    /// Consume the wrapper, returning the lower-level matchbox types.
    pub fn into_parts(
        self,
    ) -> (WebRtcSocket<MultipleChannels>, MessageLoopFuture) {
        self.builder.build()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionAddr {
    /// Connect to a local signaling server.
    Local { port: u16 },
    /// Connect to a remote signaling server.
    Remote { ip: IpAddr, port: u16 },
}

impl ConnectionAddr {
    pub fn to_url(&self) -> String {
        match self {
            ConnectionAddr::Local { port } => {
                format!("ws://0.0.0.0:{port}/")
            }
            ConnectionAddr::Remote { ip, port } => {
                format!("ws://{ip}:{port}/")
            }
        }
    }
}
