use crate::{
    events::SocketRecvEvent,
    latency::LatencyTracerPayload,
    socket::{common_socket_reader, RtcSocket},
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use instant::Duration;
use std::net::Ipv4Addr;

use super::{systems, AddProtocolExt, RtcServerEvent, RtcServerState, RtcServerStatus};

/// A plugin to serve a WebRTC server.
pub struct RtcServerPlugin {
    /// Which port to serve the signaling server on
    pub port: u16,
}

impl Plugin for RtcServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SocketRecvEvent>()
            .add_event::<RtcServerEvent>()
            .add_bounded_protocol::<LatencyTracerPayload>(2)
            .init_state::<RtcServerStatus>()
            .insert_resource(RtcServerState::new(
                (Ipv4Addr::UNSPECIFIED, self.port).into(),
            ))
            .add_systems(
                Startup,
                // We start a signaling server on localhost and the first peer
                // becomes host
                (systems::init_signaling_server, systems::init_server_socket).chain(),
            )
            .add_systems(
                First,
                (
                    common_socket_reader,
                    systems::server_event_writer,
                    systems::calculate_latency,
                )
                    .chain()
                    .run_if(resource_exists::<RtcSocket>),
            )
            .add_systems(
                Update,
                (
                    systems::read_latency_tracers,
                    systems::send_latency_tracers.run_if(on_timer(Duration::from_millis(100))),
                )
                    .run_if(in_state(RtcServerStatus::Ready)),
            );
    }
}
