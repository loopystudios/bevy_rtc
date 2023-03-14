use matchbox_socket::{
    ChannelConfig, MessageLoopFuture, RtcIceServerConfig, WebRtcSocket,
    WebRtcSocketConfig,
};
use std::net::IpAddr;

pub mod packets;

/// An abstraction over [`matchbox_socket::WebRtcSocket`] to fit Tribrid's
/// Client-Server architecture.
pub struct SilkSocket {
    socket: WebRtcSocket,
    loop_fut: MessageLoopFuture,
}

impl SilkSocket {
    /// Initialize a WebRTC socket compatible with Tribrid's Client-Server
    /// architecture.
    pub fn new(config: SilkSocketConfig) -> Self {
        let webrtc_config = config.to_matchbox_config();
        let (socket, loop_fut) = WebRtcSocket::new_with_config(webrtc_config);
        Self { socket, loop_fut }
    }

    /// Consume the wrapper, returning the lower-level matchbox types.
    pub fn into_parts(self) -> (WebRtcSocket, MessageLoopFuture) {
        (self.socket, self.loop_fut)
    }
}

#[derive(Debug, Clone)]
pub enum SilkSocketConfig {
    /// Connect to a local signalling server as the self-declared host.
    LocalSignallerAsHost { port: u16 },
    /// Connect to a local signalling server as a self-declared client.
    LocalSignallerAsClient { port: u16 },
    /// Connect to a remote signalling server as the self-declared host.
    RemoteSignallerAsHost { ip: IpAddr, port: u16 },
    /// Connect to a remote signalling server as a self-declared client.
    RemoteSignallerClient { ip: IpAddr, port: u16 },
}

impl SilkSocketConfig {
    /// The index of the unreliable channel in the [`WebRtcSocket`].
    pub const UNRELIABLE_CHANNEL_INDEX: usize = 0;
    /// The index of the reliable channel in the [`WebRtcSocket`].
    pub const RELIABLE_CHANNEL_INDEX: usize = 1;

    pub fn to_matchbox_config(&self) -> WebRtcSocketConfig {
        WebRtcSocketConfig {
            room_url: match self {
                SilkSocketConfig::LocalSignallerAsHost { port } => {
                    format!("ws://0.0.0.0:{port}/Host")
                }
                SilkSocketConfig::LocalSignallerAsClient { port } => {
                    format!("ws://0.0.0.0:{port}/Client")
                }
                SilkSocketConfig::RemoteSignallerAsHost { ip, port } => {
                    format!("ws://{ip}:{port}/Host")
                }
                SilkSocketConfig::RemoteSignallerClient { ip, port } => {
                    format!("ws://{ip}:{port}/Client")
                }
            },
            ice_server: RtcIceServerConfig::default(),
            channels: vec![
                // The ordering NEEDS to match UNRELIABLE_CHANNEL_INDEX!
                ChannelConfig::unreliable(),
                // The ordering NEEDS to match RELIABLE_CHANNEL_INDEX!
                ChannelConfig::reliable(),
            ],
            attempts: Some(3),
        }
    }
}
