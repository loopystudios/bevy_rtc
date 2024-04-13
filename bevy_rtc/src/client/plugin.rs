use super::{
    systems, AddClientProtocolExt, RtcClientEvent, RtcClientRequestEvent, RtcClientState,
    RtcClientStatus,
};
use crate::{
    events::SocketRecvEvent,
    latency::LatencyTracerPayload,
    socket::{common_socket_reader, RtcSocket},
    transport_encoding::TransportEncoding,
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use instant::Duration;

/// A plugin to connect to a WebRTC server.
pub struct RtcClientPlugin {
    /// The primary transport encoding for all packets. These are activated by cargo features.
    ///
    /// # Available encodings:
    /// - JSON: with the `json` cargo feature
    /// - Binary: with the `binary` cargo feature
    pub encoding: TransportEncoding,
}

impl Plugin for RtcClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.encoding)
            .add_event::<SocketRecvEvent>()
            .insert_resource(RtcClientState::default())
            .add_client_rw_protocol::<LatencyTracerPayload>(2)
            .init_state::<RtcClientStatus>()
            .add_event::<RtcClientRequestEvent>()
            .add_event::<RtcClientEvent>()
            .add_systems(OnEnter(RtcClientStatus::Establishing), systems::init_socket)
            .add_systems(
                OnEnter(RtcClientStatus::Disconnected),
                systems::reset_socket,
            )
            .add_systems(First, systems::connection_request_handler)
            .add_systems(
                First,
                (common_socket_reader, systems::client_event_writer)
                    .chain()
                    .run_if(resource_exists::<RtcSocket>),
            )
            .add_systems(
                First,
                systems::calculate_latency
                    .after(systems::client_event_writer)
                    .run_if(in_state(RtcClientStatus::Connected)),
            )
            .add_systems(
                Update,
                (
                    systems::read_latency_tracers,
                    systems::send_latency_tracers.run_if(on_timer(Duration::from_millis(100))),
                )
                    .run_if(in_state(RtcClientStatus::Connected)),
            );
    }
}
