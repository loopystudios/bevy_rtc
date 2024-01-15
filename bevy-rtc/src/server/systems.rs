use super::{
    events::RtcServerEvent, NetworkReader, NetworkWriter, RtcServerStatus,
    RtcState,
};
use crate::{
    latency::{LatencyTracer, LatencyTracerPayload},
    socket::RtcSocket,
};
use bevy::prelude::*;
use bevy_matchbox::{
    matchbox_signaling::{
        topologies::client_server::{ClientServer, ClientServerState},
        SignalingServerBuilder,
    },
    matchbox_socket::{PeerState, WebRtcSocket},
    prelude::ChannelConfig,
    OpenSocketExt, StartServerExt,
};
use instant::Duration;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Initialize the signaling server
pub fn init_signaling_server(mut commands: Commands, rtc_state: Res<RtcState>) {
    let host_ready: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let builder = SignalingServerBuilder::new(
        rtc_state.addr,
        ClientServer,
        ClientServerState::default(),
    )
    .on_id_assignment(|(socket, id)| info!("{socket} assigned {id}"))
    .on_host_connected({
        let addr = rtc_state.addr;
        let host_ready = host_ready.clone();
        move |id| {
            host_ready.store(true, Ordering::Relaxed);
            info!("Host ready: {id}");
            info!("Ready for clients (broadcasting on {addr})");
        }
    })
    .on_host_disconnected(|id| panic!("Host left: {id}"))
    .on_client_connected(|id| info!("Client joined: {id}"))
    .on_client_disconnected(|id| info!("Client left: {id}"))
    .on_connection_request({
        // The bevy_matchbox signaling server assigns the first connected
        // peer as host/server. As a result, we deny all connections until a
        // loopback (localhost) address has successfully connected. This
        // loopback address is ourselves, and that logic is in
        // `init_server_socket` below.
        let ready = host_ready.clone();
        move |request| {
            if ready.load(Ordering::Relaxed) {
                Ok(true)
            } else {
                let origin = request.origin.ip();
                match origin {
                    std::net::IpAddr::V4(ip) => {
                        if ip.is_loopback() {
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    }
                    std::net::IpAddr::V6(ip) => {
                        if ip.is_loopback() {
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    }
                }
            }
        }
    })
    .cors()
    .trace();
    commands.start_server(builder);
}

/// Initialize the server socket
pub fn init_server_socket(mut commands: Commands, state: Res<RtcState>) {
    // Create matchbox socket
    let room_url = format!("ws://{}", state.addr);
    let socker_builder = WebRtcSocket::builder(room_url)
        // Match UNRELIABLE_CHANNEL_INDEX
        .add_channel(ChannelConfig {
            ordered: true,
            max_retransmits: Some(0),
        })
        // Match RELIABLE_CHANNEL_INDEX
        .add_channel(ChannelConfig::reliable());
    commands.open_socket(socker_builder);
}

/// Translates socket events into Bevy events
pub fn server_event_writer(
    mut commands: Commands,
    tracer_query: Query<(Entity, &LatencyTracer)>,
    mut state: ResMut<RtcState>,
    mut socket: ResMut<RtcSocket>,
    mut event_wtr: EventWriter<RtcServerEvent>,
    mut next_server_status: ResMut<NextState<RtcServerStatus>>,
) {
    // Id changed events
    if let Some(id) = socket.id() {
        if state.id.is_none() {
            state.id.replace(id);
            event_wtr.send(RtcServerEvent::IdAssigned(id));
            next_server_status.set(RtcServerStatus::Ready);
        }
    }

    // Check for peer updates
    for (peer, peer_state) in socket.update_peers() {
        match peer_state {
            PeerState::Connected => {
                state.peers.insert(peer);
                commands.spawn(LatencyTracer::new(peer));
                event_wtr.send(RtcServerEvent::ClientJoined(peer));
            }
            PeerState::Disconnected => {
                state.peers.remove(&peer);
                if let Some((entity, _)) = tracer_query
                    .iter()
                    .find(|(_, tracer)| tracer.peer_id == peer)
                {
                    commands.entity(entity).despawn();
                }
                event_wtr.send(RtcServerEvent::ClientLeft(peer));
            }
        }
    }
}

pub fn send_latency_tracers(
    state: Res<RtcState>,
    mut writer: NetworkWriter<LatencyTracerPayload>,
) {
    let peer_id = state.id.expect("expected peer id");
    writer.unreliable_to_all(LatencyTracerPayload::new(peer_id));
}

pub fn read_latency_tracers(
    state: Res<RtcState>,
    mut tracers: Query<&mut LatencyTracer>,
    mut reader: NetworkReader<LatencyTracerPayload>,
    mut writer: NetworkWriter<LatencyTracerPayload>,
) {
    let host_id = state.id.expect("expected host id");

    // Handle payloads
    for (from, payload) in reader.read() {
        // 2 cases:
        // 1) We sent a tracer to the client, and are receiving it
        // 2) The client sent a tracer to us, and expect it back
        if payload.from == host_id {
            // Case 1
            if let Some(mut tracer) =
                tracers.iter_mut().find(|tracer| tracer.peer_id == from)
            {
                tracer.process(payload);
            }
        } else if payload.from == from {
            // Case 2
            writer.unreliable_to_peer(from, payload);
        } else {
            warn!("Invalid latency tracer from {from}: {payload:?}, ignoring");
        }
    }
}

pub fn calculate_latency(
    time: Res<Time>,
    mut state: ResMut<RtcState>,
    mut tracers: Query<&mut LatencyTracer>,
) {
    // Set latencies
    for mut tracer in tracers.iter_mut() {
        if !state.peers.contains(&tracer.peer_id) {
            state.latencies.remove(&tracer.peer_id);
            state.smoothed_latencies.remove(&tracer.peer_id);
            continue;
        }
        tracer.update_latency();

        let last_latency = tracer.last_latency.map(Duration::from_secs_f32);
        match last_latency {
            Some(last_latency) => {
                state.latencies.insert(tracer.peer_id, Some(last_latency));
                // Calculate smooth latency
                let current_smoothed = state
                    .smoothed_latencies
                    .entry(tracer.peer_id)
                    .or_insert(Some(last_latency))
                    .get_or_insert(last_latency);
                const AVG_SECS: f32 = 1.0; // 1 second average
                let alpha = 1.0 - f32::exp(-time.delta_seconds() / AVG_SECS);
                let current_f32 =
                    current_smoothed.as_secs_f32() * (1.0 - alpha);
                let delta = last_latency.as_secs_f32() * alpha;
                *current_smoothed =
                    Duration::from_secs_f32(current_f32 + delta);
            }
            None => {
                state.latencies.insert(tracer.peer_id, None);
                state.smoothed_latencies.insert(tracer.peer_id, None);
            }
        }
    }
}
