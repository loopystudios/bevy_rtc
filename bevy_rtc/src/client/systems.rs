use super::{
    events::{RtcClientEvent, RtcClientRequestEvent},
    state::{RtcClientState, RtcClientStatus},
    RtcClient,
};
use crate::{
    latency::{LatencyTracer, LatencyTracerPayload},
    socket::{RtcSocket, RtcSocketPlurality},
};
use bevy::prelude::*;
use bevy_matchbox::{
    matchbox_socket::{self, WebRtcSocket},
    prelude::*,
};
use instant::Duration;

/// Initialize the socket
pub(crate) fn init_socket(mut commands: Commands, socket_res: Res<RtcClientState>) {
    if let Some(addr) = socket_res.addr.as_ref() {
        debug!("connecting to: {addr:?}");

        // Create matchbox socket
        let socker_builder = WebRtcSocket::builder(addr)
            // Match UNRELIABLE_CHANNEL_INDEX
            .add_channel(ChannelConfig {
                ordered: true,
                max_retransmits: Some(0),
            })
            // Match RELIABLE_CHANNEL_INDEX
            .add_channel(ChannelConfig::reliable());

        // Open socket
        commands.open_socket(socker_builder);
    } else {
        panic!("state set to connecting without config");
    }
}

/// Reset the internal socket
pub(crate) fn reset_socket(
    mut commands: Commands,
    tracer_query: Query<Entity, With<LatencyTracer>>,
    mut state: ResMut<RtcClientState>,
) {
    commands.close_socket::<RtcSocketPlurality>();
    if let Ok(entity) = tracer_query.get_single() {
        commands.entity(entity).despawn();
    }
    *state = RtcClientState {
        // Keep for reconnecting
        addr: state.addr.clone(),
        host_peer_id: None,
        peer_id: None,
        latency: None,
        smoothed_latency: None,
    };
}

/// Reads and handles connection request events
pub(crate) fn connection_request_handler(
    mut request_reader: EventReader<RtcClientRequestEvent>,
    mut state: ResMut<RtcClientState>,
    mut next_connection_state: ResMut<NextState<RtcClientStatus>>,
    current_connection_state: Res<State<RtcClientStatus>>,
    mut event_wtr: EventWriter<RtcClientEvent>,
) {
    match request_reader.read().next() {
        Some(RtcClientRequestEvent::Connect { addr }) => {
            if let RtcClientStatus::Disconnected = current_connection_state.get() {
                debug!(
                    previous = format!("{current_connection_state:?}"),
                    "set state: connecting"
                );
                state.addr.replace(addr.to_owned());
                next_connection_state.set(RtcClientStatus::Establishing);
            }
        }
        Some(RtcClientRequestEvent::Disconnect) => {
            debug!(
                previous = format!("{current_connection_state:?}"),
                "set state: disconnected"
            );
            next_connection_state.set(RtcClientStatus::Disconnected);
            event_wtr.send(RtcClientEvent::DisconnectedFromHost {
                reason: Some("Client requested to disconnect".to_string()),
            });
        }
        None => {}
    }
}

/// Translates socket updates into bevy events
pub(crate) fn client_event_writer(
    mut commands: Commands,
    mut state: ResMut<RtcClientState>,
    mut socket: ResMut<RtcSocket>,
    mut event_wtr: EventWriter<RtcClientEvent>,
    mut next_connection_state: ResMut<NextState<RtcClientStatus>>,
) {
    // Create events

    // Id changed events
    if let Some(peer_id) = socket.id() {
        if state.peer_id.is_none() {
            state.peer_id.replace(peer_id);
            event_wtr.send(RtcClientEvent::IdAssigned(peer_id));
        }
    }

    // Connection state updates
    match socket.try_update_peers() {
        Ok(updates) => {
            for (id, peer_state) in updates {
                match peer_state {
                    matchbox_socket::PeerState::Connected => {
                        state.host_peer_id.replace(id);
                        commands.spawn(LatencyTracer::new(id));
                        next_connection_state.set(RtcClientStatus::Connected);
                        event_wtr.send(RtcClientEvent::ConnectedToHost(id));
                    }
                    matchbox_socket::PeerState::Disconnected => {
                        next_connection_state.set(RtcClientStatus::Disconnected);
                        event_wtr.send(RtcClientEvent::DisconnectedFromHost {
                            reason: Some("Server reset".to_string()),
                        });
                    }
                }
            }
        }
        Err(e) => {
            error!("read channel error: {e:?}");
        }
    }

    if socket.any_closed() {
        next_connection_state.set(RtcClientStatus::Disconnected);
        event_wtr.send(RtcClientEvent::DisconnectedFromHost {
            reason: Some("Connection closed".to_string()),
        });
    }
}

pub fn send_latency_tracers(
    state: Res<RtcClientState>,
    mut client: RtcClient<LatencyTracerPayload>,
) {
    let peer_id = state.peer_id.expect("expected peer id");
    client.unreliable_to_host(LatencyTracerPayload::new(peer_id));
}

pub fn read_latency_tracers(
    state: Res<RtcClientState>,
    mut trace_query: Query<&mut LatencyTracer>,
    mut client: RtcClient<LatencyTracerPayload>,
) {
    let host_id = state.host_peer_id.expect("expected host id");
    let peer_id = state.peer_id.expect("expected peer id");
    let mut tracer = trace_query.single_mut();

    for payload in client.read() {
        if payload.from == peer_id {
            tracer.process(payload);
        } else if payload.from == host_id {
            // Server time payloads get sent right back to the server
            client.unreliable_to_host(payload);
        }
        // Process payloads we sent out
        else {
            warn!(
                "Invalid latency tracer from address: {}, ignoring",
                payload.from
            );
        }
    }
}

pub fn calculate_latency(
    time: Res<Time>,
    mut state: ResMut<RtcClientState>,
    mut tracer: Query<&mut LatencyTracer>,
) {
    let mut tracer = tracer.single_mut();
    tracer.update_latency();

    let last_latency = tracer.last_latency.map(Duration::from_secs_f32);
    match last_latency {
        Some(last_latency) => {
            state.latency.replace(last_latency);
            let current_smoothed = state.smoothed_latency.get_or_insert(last_latency);
            const AVG_SECS: f32 = 1.0; // 1 second average
            let alpha = 1.0 - f32::exp(-time.delta_seconds() / AVG_SECS);
            let current_f32 = current_smoothed.as_secs_f32() * (1.0 - alpha);
            let delta = last_latency.as_secs_f32() * alpha;
            *current_smoothed = Duration::from_secs_f32(current_f32 + delta);
        }
        None => {
            state.latency = None;
            state.smoothed_latency = None;
        }
    }
}
