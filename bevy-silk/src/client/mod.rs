use crate::{
    events::SocketRecvEvent,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::{SilkSchedule, SilkSet},
    socket::{common_socket_reader, SilkSocket},
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
        app.init_schedule(SilkSchedule)
            .edit_schedule(SilkSchedule, |schedule| {
                schedule.configure_sets(SilkSet::sets());
            })
            .add_systems(Update, |world: &mut World| {
                world.run_schedule(SilkSchedule);
            })
            .add_event::<SocketRecvEvent>()
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .insert_resource(SilkState::default())
            .add_state::<SilkConnectionState>()
            .add_event::<ConnectionRequest>()
            .add_event::<SilkClientEvent>()
            .add_systems(
                OnEnter(SilkConnectionState::Establishing),
                systems::init_socket,
            )
            .add_systems(
                OnEnter(SilkConnectionState::Disconnected),
                systems::reset_socket,
            )
            .add_systems(Update, systems::connection_request_handler)
            .add_systems(
                SilkSchedule,
                (
                    common_socket_reader,
                    systems::client_event_writer,
                    systems::on_login,
                )
                    .chain()
                    .run_if(resource_exists::<SilkSocket>())
                    .before(SilkSet::PreUpdate),
            );
    }
}
