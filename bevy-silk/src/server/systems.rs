use super::{events::SilkServerEvent, SilkState};
use crate::socket::SilkSocket;
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
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Initialize the signaling server
pub fn init_signaling_server(
    mut commands: Commands,
    silk_state: Res<SilkState>,
) {
    let host_ready: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let builder = SignalingServerBuilder::new(
        silk_state.addr,
        ClientServer,
        ClientServerState::default(),
    )
    .on_id_assignment(|(socket, id)| info!("{socket} assigned {id}"))
    .on_host_connected({
        let addr = silk_state.addr;
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
pub fn init_server_socket(mut commands: Commands, state: Res<SilkState>) {
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
    mut state: ResMut<SilkState>,
    mut socket: ResMut<SilkSocket>,
    mut event_wtr: EventWriter<SilkServerEvent>,
) {
    // Id changed events
    if let Some(id) = socket.id() {
        if state.id.is_none() {
            state.id.replace(id);
            event_wtr.send(SilkServerEvent::IdAssigned(id));
        }
    }

    // Check for peer updates
    for (peer, peer_state) in socket.update_peers() {
        match peer_state {
            PeerState::Connected => {
                event_wtr.send(SilkServerEvent::ClientJoined(peer));
            }
            PeerState::Disconnected => {
                event_wtr.send(SilkServerEvent::ClientLeft(peer));
            }
        }
    }
}
