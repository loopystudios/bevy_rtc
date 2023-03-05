use matchbox_socket::{
    ChannelConfig, Error, RtcIceServerConfig, WebRtcSocket, WebRtcSocketConfig,
};
use std::{future::Future, net::IpAddr, pin::Pin};

pub mod packets;

#[cfg(not(target_arch = "wasm32"))]
type MessageLoopFuture =
    Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;
#[cfg(target_arch = "wasm32")]
type MessageLoopFuture = Pin<Box<dyn Future<Output = Result<(), Error>>>>;

pub struct SilkSocket {
    socket: WebRtcSocket,
    loop_fut: MessageLoopFuture,
}

impl SilkSocket {
    pub fn new(config: SilkSocketConfig) -> Self {
        let webrtc_config = config.to_matchbox_config();
        let (socket, loop_fut) = WebRtcSocket::new_with_config(webrtc_config);
        Self { socket, loop_fut }
    }

    pub fn into_parts(self) -> (WebRtcSocket, MessageLoopFuture) {
        (self.socket, self.loop_fut)
    }
}

#[derive(Debug, Clone)]
pub enum SilkSocketConfig {
    LocalSignallerAsHost { port: u16 },
    LocalSignallerAsClient { port: u16 },
    RemoteSignallerAsHost { ip: IpAddr, port: u16 },
    RemoteSignallerClient { ip: IpAddr, port: u16 },
}

impl SilkSocketConfig {
    pub const UNRELIABLE_CHANNEL_INDEX: usize = 0;
    pub const RELIABLE_CHANNEL_INDEX: usize = 1;

    pub fn to_matchbox_config(&self) -> WebRtcSocketConfig {
        WebRtcSocketConfig {
            room_url: match self {
                SilkSocketConfig::LocalSignallerAsHost { port } => {
                    format!("ws://localhost:{port}/Host")
                }
                SilkSocketConfig::LocalSignallerAsClient { port } => {
                    format!("ws://localhost:{port}/Client")
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
                ChannelConfig::unreliable(),
                ChannelConfig::reliable(),
            ],
            attempts: Some(3),
        }
    }
}

pub type Packet = Box<[u8]>;
