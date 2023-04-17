use bevy::{prelude::*, time::fixed_timestep::FixedTime};
pub use router::{AddNetworkMessageExt, IncomingMessages, OutgoingMessages};
use signaler::SilkSignalerPlugin;
use silk_common::{
    events::SilkServerEvent,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::*,
    ConnectionAddr, SilkCommonPlugin, SilkStage,
};
use state::SocketState;
pub use system_params::{ServerRecv, ServerSend};

mod router;
pub mod signaler;
pub(crate) mod state;
mod system_params;
pub(crate) mod systems;

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Whether the signaling server is local or remote
    pub signaler_addr: ConnectionAddr,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f32,
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        if let ConnectionAddr::Local { port } = self.signaler_addr {
            app.add_plugin(SilkSignalerPlugin { port });
        }

        app.add_plugin(SilkCommonPlugin)
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .add_event::<SilkServerEvent>()
            .insert_resource(SocketState {
                addr: self.signaler_addr,
                id: None,
            })
            .add_startup_system(systems::init_socket)
            .insert_resource(FixedTime::new_from_secs(1.0 / self.tick_rate));

        app.add_system(
            trace_read
                .before(systems::on_login)
                .before(systems::server_socket_reader)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            systems::on_login
                .after(SilkStage::NetworkRead)
                .before(SilkStage::SilkEvents)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            // Read silk events always before servers, who hook into this stage
            systems::server_socket_reader
                .before(SilkStage::SilkEvents)
                .after(systems::on_login)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            trace_incoming
                .after(SilkStage::SilkEvents)
                .before(SilkStage::Process)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            trace_update_state
                .after(SilkStage::Process)
                .before(SilkStage::Update)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            trace_write
                .after(SilkStage::Update)
                .before(SilkStage::NetworkWrite)
                .in_schedule(SilkSchedule),
        );
    }
}

fn trace_read() {
    trace!("Trace 1: Read");
}

fn trace_incoming() {
    trace!("Trace 2: Latency Processing");
}

fn trace_update_state() {
    trace!("Trace 3: Update");
}

fn trace_write() {
    trace!("Trace 4: Write");
}
