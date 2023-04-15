mod signaler;
mod state;
mod systems;

use bevy::{prelude::*, time::fixed_timestep::FixedTime};
use events::{SilkBroadcastEvent, SilkServerEvent};
use schedule::{SilkServerSchedule, SilkServerStage};
use signaler::SilkSignalerPlugin;
use silk_common::ConnectionAddr;
use state::SocketState;

pub mod events;
pub mod schedule;

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Whether the signaling server is local or remote
    pub signaler_addr: ConnectionAddr,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f32,
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        if let ConnectionAddr::Local { port, .. } = self.signaler_addr {
            app.add_plugin(SilkSignalerPlugin { port });
        }
        app.insert_resource(SocketState {
            addr: self.signaler_addr,
            id: None,
        })
        .add_startup_system(systems::init_socket)
        .insert_resource(FixedTime::new_from_secs(1.0 / self.tick_rate))
        .add_event::<SilkServerEvent>()
        .add_event::<SilkBroadcastEvent>();

        app.init_schedule(SilkServerSchedule);

        // it's important here to configure set order
        app.edit_schedule(SilkServerSchedule, |schedule| {
            schedule.configure_sets(SilkServerStage::sets());
        });

        app.add_system(
            trace_read
                .before(systems::socket_reader)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            // Read silk events always before servers, who hook into this stage
            systems::socket_reader
                .before(SilkServerStage::ReadIn)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            trace_incoming
                .after(SilkServerStage::ReadIn)
                .before(SilkServerStage::ProcessLatency)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            trace_update_state
                .after(SilkServerStage::ProcessLatency)
                .before(SilkServerStage::Update)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            trace_write
                .after(SilkServerStage::Update)
                .before(SilkServerStage::WriteOut)
                .in_schedule(SilkServerSchedule),
        )
        .add_system(
            // Write silk events always after servers, who hook into this stage
            systems::broadcast
                .after(SilkServerStage::WriteOut)
                .in_schedule(SilkServerSchedule),
        );

        // add scheduler
        app.add_system(
            schedule::run_silk_schedule.in_schedule(CoreSchedule::FixedUpdate),
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
