use bevy::{prelude::*, time::fixed_timestep::FixedTime};
use signaler::SilkSignalerPlugin;
use silk_common::events::SilkServerEvent;
use silk_common::packets::auth::{SilkAuthGuestPayload, SilkAuthUserPayload};
use silk_common::{schedule::*, SilkCommonPlugin, SilkStage};
use silk_common::{AddNetworkMessageExt, ConnectionAddr};
use state::SocketState;

pub mod signaler;
pub(crate) mod state;
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
            .add_event::<SilkServerEvent>()
            .insert_resource(SocketState {
                addr: self.signaler_addr,
                id: None,
            })
            .add_startup_system(systems::init_socket)
            .insert_resource(FixedTime::new_from_secs(1.0 / self.tick_rate));

        app.add_system(
            trace_read
                .before(systems::socket_reader)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            // Read silk events always before servers, who hook into this stage
            systems::socket_reader
                .before(SilkStage::ReadIn)
                .in_schedule(SilkSchedule),
        )
        .add_systems(
            (systems::on_login, systems::on_guest_login)
                .in_base_set(SilkStage::ReadIn)
                .in_schedule(SilkSchedule),
        )
        .add_system(
            trace_incoming
                .after(SilkStage::ReadIn)
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
                .before(SilkStage::WriteOut)
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
