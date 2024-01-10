use crate::{
    events::SocketRecvEvent,
    latency::LatencyTracerPayload,
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
pub use state::{SilkClientStatus, SilkState};
pub use system_params::{NetworkReader, NetworkWriter};

/// The socket client abstraction
pub struct SilkClientPlugin;

impl Plugin for SilkClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SocketRecvEvent>()
            .insert_resource(SilkState::default())
            .add_network_message::<LatencyTracerPayload>()
            .add_state::<SilkClientStatus>()
            .add_event::<ConnectionRequest>()
            .add_event::<SilkClientEvent>()
            .add_systems(
                OnEnter(SilkClientStatus::Establishing),
                systems::init_socket,
            )
            .add_systems(
                OnEnter(SilkClientStatus::Disconnected),
                systems::reset_socket,
            )
            .add_systems(First, systems::connection_request_handler)
            .add_systems(
                First,
                (common_socket_reader, systems::client_event_writer)
                    .chain()
                    .run_if(resource_exists::<SilkSocket>()),
            )
            .add_systems(
                First,
                systems::update_state_latency
                    .after(systems::client_event_writer)
                    .run_if(state_exists_and_equals(
                        SilkClientStatus::Connected,
                    )),
            )
            .add_systems(
                Update,
                (systems::read_latency_tracers, systems::send_latency_tracers)
                    .run_if(state_exists_and_equals(
                        SilkClientStatus::Connected,
                    )),
            );
    }
}
