use bevy::prelude::*;
use bevy_matchbox::matchbox_socket::{
    ChannelConfig, MessageLoopFuture, MultipleChannels, WebRtcSocket,
    WebRtcSocketBuilder,
};
use events::SocketRecvEvent;
use schedule::SilkSchedule;
use socket::common_socket_reader;
use std::net::IpAddr;

pub mod demo_packets;
pub mod events;
pub mod packets;
pub mod schedule;
pub mod socket;
pub mod stage;

// Re-exports
pub use bevy_matchbox;
pub use stage::SilkStage;
pub mod macros {
    pub use proc_macro_payload::Payload;
}

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

#[derive(Debug, Clone)]
pub enum AuthenticationRequest {
    Registered {
        access_token: String,
        character: String,
    },
    Guest {
        username: Option<String>,
    },
}
impl Default for AuthenticationRequest {
    fn default() -> Self {
        AuthenticationRequest::Guest { username: None }
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

        app.add_event::<SocketRecvEvent>()
            .add_system(
                trace_flush
                    .before(SilkStage::Flush)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                trace_network_read
                    .after(SilkStage::Flush)
                    .before(SilkStage::NetworkRead)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                // Read silk events always before servers, who hook into this
                // stage
                common_socket_reader
                    .after(trace_network_read)
                    .before(SilkStage::NetworkRead)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                trace_process
                    .after(SilkStage::NetworkRead)
                    .before(SilkStage::Process)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                trace_silk_events
                    .after(SilkStage::Process)
                    .before(SilkStage::SilkEvents)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                trace_pre_update
                    .after(SilkStage::SilkEvents)
                    .before(SilkStage::PreUpdate)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                trace_update
                    .after(SilkStage::PreUpdate)
                    .before(SilkStage::Update)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                trace_post_update
                    .after(SilkStage::Update)
                    .before(SilkStage::PostUpdate)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                trace_network_write
                    .after(SilkStage::PostUpdate)
                    .before(SilkStage::NetworkWrite)
                    .in_schedule(SilkSchedule),
            );

        // add scheduler
        app.add_system(
            schedule::run_silk_schedule.in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

fn trace_flush() {
    trace!("System start: {}", SilkStage::Flush);
}

fn trace_network_read() {
    trace!("System start: {}", SilkStage::NetworkRead);
}

fn trace_process() {
    trace!("System start: {}", SilkStage::Process);
}

fn trace_silk_events() {
    trace!("System start: {}", SilkStage::SilkEvents);
}

fn trace_pre_update() {
    trace!("System start: {}", SilkStage::PreUpdate);
}

fn trace_update() {
    trace!("System start: {}", SilkStage::Update);
}

fn trace_post_update() {
    trace!("System start: {}", SilkStage::PostUpdate);
}

fn trace_network_write() {
    trace!("System start: {}", SilkStage::NetworkWrite);
}
