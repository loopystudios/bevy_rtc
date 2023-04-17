mod state;
mod systems;

use bevy::prelude::*;
use events::ConnectionRequest;
pub use router::{AddNetworkMessageExt, IncomingMessages, OutgoingMessages};
use silk_common::events::SilkClientEvent;
use silk_common::packets::auth::{
    SilkLoginRequestPayload, SilkLoginResponsePayload,
};
use silk_common::schedule::SilkSchedule;
use silk_common::{SilkCommonPlugin, SilkStage};
use state::{ClientState, ConnectionState};
pub use system_params::{ClientRecv, ClientSend};

pub mod events;
mod router;
mod system_params;

/// The socket client abstraction
pub struct SilkClientPlugin;

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SilkCommonPlugin)
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .insert_resource(ClientState::default())
            .add_state::<ConnectionState>()
            .add_event::<ConnectionRequest>()
            .add_event::<SilkClientEvent>();

        app.add_system(systems::connection_event_reader)
            .add_system(
                systems::init_socket
                    .in_schedule(OnEnter(ConnectionState::Establishing)),
            )
            .add_system(
                systems::reset_socket
                    .in_schedule(OnEnter(ConnectionState::Disconnected)),
            )
            .add_system(
                trace_read
                    .before(systems::socket_reader)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                systems::socket_reader
                    .before(SilkStage::ReadIn)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                systems::on_login_accepted
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
