use super::{
    events::{ConnectionRequest, SilkClientEvent},
    state::{SilkClientStatus, SilkState},
    NetworkReader, NetworkWriter,
};
use crate::{
    latency::{LatencyTracer, LatencyTracerPayload},
    socket::{SilkSocket, SilkSocketPlurality},
};
use bevy::prelude::*;
use bevy_matchbox::{
    matchbox_socket::{self, WebRtcSocket},
    prelude::*,
};
use instant::Duration;

/// Initialize the socket
pub(crate) fn init_socket(mut commands: Commands, socket_res: Res<SilkState>) {
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
    mut state: ResMut<SilkState>,
) {
    commands.close_socket::<SilkSocketPlurality>();
    if let Ok(entity) = tracer_query.get_single() {
        commands.entity(entity).despawn();
    }
    *state = SilkState {
        // Keep for reconnecting
        addr: state.addr.clone(),
        host_id: None,
        id: None,
        latency: None,
        smoothed_latency: None,
    };
}

/// Reads and handles connection request events
pub(crate) fn connection_request_handler(
    mut cxn_event_reader: EventReader<ConnectionRequest>,
    mut state: ResMut<SilkState>,
    mut next_connection_state: ResMut<NextState<SilkClientStatus>>,
    current_connection_state: Res<State<SilkClientStatus>>,
    mut event_wtr: EventWriter<SilkClientEvent>,
) {
    match cxn_event_reader.read().next() {
        Some(ConnectionRequest::Connect { addr }) => {
            if let SilkClientStatus::Disconnected =
                current_connection_state.get()
            {
                debug!(
                    previous = format!("{current_connection_state:?}"),
                    "set state: connecting"
                );
                state.addr.replace(addr.to_owned());
                next_connection_state.set(SilkClientStatus::Establishing);
            }
        }
        Some(ConnectionRequest::Disconnect) => {
            debug!(
                previous = format!("{current_connection_state:?}"),
                "set state: disconnected"
            );
            next_connection_state.set(SilkClientStatus::Disconnected);
            event_wtr.send(SilkClientEvent::DisconnectedFromHost {
                reason: Some("Client requested to disconnect".to_string()),
            });
        }
        None => {}
    }
}

/// Translates socket updates into bevy events
pub(crate) fn client_event_writer(
    mut commands: Commands,
    mut state: ResMut<SilkState>,
    mut socket: ResMut<SilkSocket>,
    mut event_wtr: EventWriter<SilkClientEvent>,
    mut next_connection_state: ResMut<NextState<SilkClientStatus>>,
) {
    // Create socket events for Silk

    // Id changed events
    if let Some(id) = socket.id() {
        if state.id.is_none() {
            state.id.replace(id);
            event_wtr.send(SilkClientEvent::IdAssigned(id));
        }
    }

    // Connection state updates
    match socket.try_update_peers() {
        Ok(updates) => {
            for (id, peer_state) in updates {
                match peer_state {
                    matchbox_socket::PeerState::Connected => {
                        state.host_id.replace(id);
                        commands.spawn(LatencyTracer::new(id));
                        next_connection_state.set(SilkClientStatus::Connected);
                        event_wtr.send(SilkClientEvent::ConnectedToHost(id));
                    }
                    matchbox_socket::PeerState::Disconnected => {
                        next_connection_state
                            .set(SilkClientStatus::Disconnected);
                        event_wtr.send(SilkClientEvent::DisconnectedFromHost {
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
        next_connection_state.set(SilkClientStatus::Disconnected);
        event_wtr.send(SilkClientEvent::DisconnectedFromHost {
            reason: Some("Connection closed".to_string()),
        });
    }
}

pub fn send_latency_tracers(
    state: Res<SilkState>,
    mut writer: NetworkWriter<LatencyTracerPayload, 100>,
) {
    let peer_id = state.id.expect("expected peer id");
    writer.unreliable_to_host_with(|| LatencyTracerPayload::new(peer_id));
}

pub fn read_latency_tracers(
    state: Res<SilkState>,
    mut trace_query: Query<&mut LatencyTracer>,
    mut reader: NetworkReader<LatencyTracerPayload>,
    mut writer: NetworkWriter<LatencyTracerPayload>,
) {
    let host_id = state.host_id.expect("expected host id");
    let peer_id = state.id.expect("expected peer id");
    let mut tracer = trace_query.single_mut();

    // Only collect the most recent payloads that happens this tick.
    let mut most_recent_payload: Option<LatencyTracerPayload> = None;

    for payload in reader.read() {
        // Server time payloads get sent right back to the server
        if payload.from == host_id {
            if let Some(ref mrp) = most_recent_payload {
                if mrp.age() > payload.age() {
                    most_recent_payload.replace(payload);
                }
            } else {
                most_recent_payload.replace(payload);
            }
        }
        // Process payloads we sent out
        else if payload.from == peer_id {
            tracer.process(payload.clone());
        } else {
            warn!(
                "Invalid latency tracer from address: {}, ignoring",
                payload.from
            );
        }
    }

    // Send all server requests
    if let Some(payload) = most_recent_payload.take() {
        writer.unreliable_to_host(payload);
    }
}

pub fn calculate_latency(
    time: Res<Time>,
    mut state: ResMut<SilkState>,
    mut tracer: Query<&mut LatencyTracer>,
) {
    let mut tracer = tracer.single_mut();
    tracer.update_latency();

    let last_latency = tracer.last_latency.map(Duration::from_secs_f32);
    match last_latency {
        Some(last_latency) => {
            state.latency.replace(last_latency);
            let current_smoothed =
                state.smoothed_latency.get_or_insert(last_latency);
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
