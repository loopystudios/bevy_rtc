use crate::{
    common_plugin::SilkCommonPlugin,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::{SilkSchedule, SilkSet},
    socket::SilkSocket,
};
use bevy::prelude::*;

mod events;
mod router;
mod state;
mod system_params;
mod systems;

pub use events::{ConnectionRequest, SilkClientEvent};
pub use router::AddNetworkMessageExt;
pub use state::{SilkConnectionState, SilkState};
pub use system_params::{NetworkReader, NetworkWriter};

/// The socket client abstraction
pub struct SilkClientPlugin;

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SilkCommonPlugin)
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .insert_resource(SilkState::default())
            .add_state::<SilkConnectionState>()
            .add_event::<ConnectionRequest>()
            .add_event::<SilkClientEvent>();

        app.add_systems(
            OnEnter(SilkConnectionState::Establishing),
            systems::init_socket,
        )
        .add_systems(
            OnEnter(SilkConnectionState::Disconnected),
            systems::reset_socket,
        )
        .add_systems(
            SilkSchedule,
            systems::client_socket_reader
                .in_set(SilkSet::NetworkRead)
                .run_if(resource_exists::<SilkSocket>()),
        )
        .add_systems(
            SilkSchedule,
            (systems::connection_event_reader, systems::on_login_accepted)
                .before(SilkSet::SilkEvents)
                .after(SilkSet::Process),
        );
    }
}
