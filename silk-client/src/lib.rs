mod state;
mod systems;

use bevy::prelude::*;
use events::ConnectionRequest;
pub use router::{AddNetworkMessageExt, IncomingMessages, OutgoingMessages};
use silk_common::{
    events::SilkClientEvent,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::SilkSchedule,
    SilkCommonPlugin, SilkStage,
};
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
                systems::client_socket_reader
                    .in_base_set(SilkStage::NetworkRead)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                systems::on_login_accepted
                    .in_base_set(SilkStage::SilkEvents)
                    .in_schedule(SilkSchedule),
            );
    }
}
