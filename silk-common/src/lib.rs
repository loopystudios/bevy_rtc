use std::net::IpAddr;

use matchbox_socket::{ChannelConfig, RtcIceServerConfig, WebRtcSocketConfig};

pub mod packets;

pub enum SocketConfig {
    LocalHost { port: u16 },
    LocalClient { port: u16 },
    RemoteHost { ip: IpAddr, port: u16 },
    RemoteClient { ip: String, port: u16 },
}

impl SocketConfig {
    pub const UNRELIABLE_CHANNEL_INDEX: usize = 0;
    pub const RELIABLE_CHANNEL_INDEX: usize = 1;

    pub fn get(&self) -> WebRtcSocketConfig {
        WebRtcSocketConfig {
            room_url: match self {
                SocketConfig::LocalHost { port } => {
                    format!("ws://localhost:{port}/Host")
                }
                SocketConfig::LocalClient { port } => {
                    format!("ws://localhost:{port}/Client")
                }
                SocketConfig::RemoteHost { ip, port } => {
                    format!("ws://{ip}:{port}/Host")
                }
                SocketConfig::RemoteClient { ip, port } => {
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
