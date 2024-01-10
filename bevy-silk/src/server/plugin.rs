use crate::{
    events::SocketRecvEvent,
    latency::LatencyTracerPayload,
    socket::{common_socket_reader, SilkSocket},
};
use bevy::prelude::*;
use std::net::Ipv4Addr;

use super::{
    systems, AddNetworkMessageExt, SilkServerEvent, SilkServerStatus, SilkState,
};

/// A plugin to serve a WebRTC server.
pub struct SilkServerPlugin {
    /// Which port to serve the signaling server on
    pub port: u16,
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the schedule for silk
        app.add_event::<SocketRecvEvent>()
            .add_event::<SilkServerEvent>()
            .add_network_message::<LatencyTracerPayload>()
            .add_state::<SilkServerStatus>()
            .insert_resource(SilkState::new(
                (Ipv4Addr::UNSPECIFIED, self.port).into(),
            ))
            .add_systems(
                Startup,
                // We start a signaling server on localhost and the first peer
                // becomes host
                (systems::init_signaling_server, systems::init_server_socket)
                    .chain(),
            )
            .add_systems(
                First,
                (
                    common_socket_reader,
                    systems::server_event_writer,
                    systems::update_state_latency,
                )
                    .chain()
                    .run_if(resource_exists::<SilkSocket>()),
            )
            .add_systems(
                Update,
                (systems::read_latency_tracers, systems::send_latency_tracers)
                    .run_if(state_exists_and_equals(SilkServerStatus::Ready)),
            );
    }
}
