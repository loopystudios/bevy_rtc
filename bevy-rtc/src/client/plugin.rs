use super::{
    systems, AddProtocolExt, ConnectionRequest, RtcClientEvent,
    RtcClientStatus, RtcState,
};
use crate::{
    events::SocketRecvEvent,
    latency::LatencyTracerPayload,
    socket::{common_socket_reader, RtcSocket},
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use instant::Duration;

/// A plugin to connect to a WebRTC server.
pub struct RtcClientPlugin;

impl Plugin for RtcClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SocketRecvEvent>()
            .insert_resource(RtcState::default())
            .add_bounded_protocol::<LatencyTracerPayload>(2)
            .add_state::<RtcClientStatus>()
            .add_event::<ConnectionRequest>()
            .add_event::<RtcClientEvent>()
            .add_systems(
                OnEnter(RtcClientStatus::Establishing),
                systems::init_socket,
            )
            .add_systems(
                OnEnter(RtcClientStatus::Disconnected),
                systems::reset_socket,
            )
            .add_systems(First, systems::connection_request_handler)
            .add_systems(
                First,
                (common_socket_reader, systems::client_event_writer)
                    .chain()
                    .run_if(resource_exists::<RtcSocket>()),
            )
            .add_systems(
                First,
                systems::calculate_latency
                    .after(systems::client_event_writer)
                    .run_if(state_exists_and_equals(
                        RtcClientStatus::Connected,
                    )),
            )
            .add_systems(
                Update,
                (
                    systems::read_latency_tracers,
                    systems::send_latency_tracers
                        .run_if(on_timer(Duration::from_millis(100))),
                )
                    .run_if(state_exists_and_equals(
                        RtcClientStatus::Connected,
                    )),
            );
    }
}
