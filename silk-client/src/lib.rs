mod state;
mod systems;

use bevy::prelude::*;
use events::ConnectionRequest;
pub use router::{AddNetworkMessageExt, IncomingMessages, OutgoingMessages};
use silk_common::{
    events::SilkClientEvent,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::SilkSchedule,
    sets::SilkSet,
    SilkCommonPlugin,
};
use state::ClientState;
pub use state::ConnectionState;
pub use system_params::{NetworkReader, NetworkWriter};

pub mod events;
mod router;
mod system_params;

/// The socket client abstraction
pub struct SilkClientPlugin;

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SilkCommonPlugin)
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .insert_resource(ClientState::default())
            .add_state::<ConnectionState>()
            .add_event::<ConnectionRequest>()
            .add_event::<SilkClientEvent>();

        app.add_systems(
            OnEnter(ConnectionState::Establishing),
            systems::init_socket,
        )
        .add_systems(
            OnEnter(ConnectionState::Disconnected),
            systems::reset_socket,
        )
        .add_systems(
            SilkSchedule,
            systems::client_socket_reader.in_set(SilkSet::NetworkRead),
        )
        .add_systems(
            SilkSchedule,
            (systems::connection_event_reader, systems::on_login_accepted)
                .before(SilkSet::SilkEvents)
                .after(SilkSet::Process),
        );
    }
}
