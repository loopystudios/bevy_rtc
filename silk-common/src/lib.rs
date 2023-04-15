use bevy::prelude::*;
use bevy_matchbox::matchbox_socket::{
    ChannelConfig, MessageLoopFuture, MultipleChannels, WebRtcSocket,
    WebRtcSocketBuilder,
};
use events::SocketRecvEvent;
use schedule::SilkSchedule;
use socket::{handle_socket_events, socket_reader, SocketState};
use std::net::IpAddr;

pub mod demo_packets;
mod events;
pub mod packets;
pub mod router;
pub mod schedule;
pub mod socket;
pub mod stage;

// Re-exports
pub use bevy_matchbox;
pub use events::SilkSocketEvent;
pub use router::AddNetworkMessage;
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

pub struct SilkCommonPlugin;

impl Plugin for SilkCommonPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(SilkSchedule);

        // it's important here to configure set order
        app.edit_schedule(SilkSchedule, |schedule| {
            schedule.configure_sets(SilkStage::sets());
        });

        app.init_resource::<SocketState>()
            .add_event::<SocketRecvEvent>()
            .add_event::<SilkSocketEvent>()
            .add_system(handle_socket_events)
            .add_system(
                // Read silk events always before servers, who hook into this stage
                socket_reader
                    .before(SilkStage::ReadIn)
                    .in_schedule(SilkSchedule),
            );

        // add scheduler
        app.add_system(
            schedule::run_silk_schedule.in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
