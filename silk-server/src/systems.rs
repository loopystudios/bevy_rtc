use crate::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    state::SocketState,
};
use bevy::prelude::*;
use silk_common::SilkSocket;
use silk_common::{
    bevy_matchbox::{
        matchbox_socket::{PeerId, PeerState},
        prelude::MultipleChannels,
        MatchboxSocket, OpenSocketExt,
    },
    packets::SilkPayload,
};

/// Initialize the socket
pub(crate) fn init_socket(mut commands: Commands, state: Res<SocketState>) {
    debug!("address: {:?}", state.addr);

    // Create matchbox socket
    let silk_socket = SilkSocket::new(state.addr);
    commands.open_socket(silk_socket.builder());
}

/// Translates socket events into Bevy events
pub(crate) fn socket_reader(
    mut state: ResMut<SocketState>,
    mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
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
            PeerState::Connected => {}
            PeerState::Disconnected => {
                event_wtr.send(SilkServerEvent::ClientLeft(peer));
            }
        }
    }

    // Collect Unreliable, Reliable messages
    let messages = socket
        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
        .receive()
        .into_iter()
        .chain(
            socket
                .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                .receive(),
        );

    // Handle auth or forward message
    for (peer_id, packet) in messages {
        if let Ok(payload) = SilkPayload::try_from(&packet) {
            trace!("received payload: {payload:?}");
            match payload {
                SilkPayload::AuthenticateUser {
                    username,
                    password,
                    mfa,
                } => {
                    debug!(username, "received login request");
                    event_wtr.send(SilkServerEvent::LoginRequest {
                        peer_id,
                        username,
                        password,
                        mfa,
                    })
                }
                SilkPayload::AuthenticateGuest { username } => {
                    debug!(username, "received guest login request");
                    event_wtr.send(SilkServerEvent::GuestLoginRequest {
                        peer_id,
                        username,
                    })
                }
                SilkPayload::Message(packet) => {
                    event_wtr.send(SilkServerEvent::Message((peer_id, packet)));
                }
                SilkPayload::LoginAccepted { .. }
                | SilkPayload::LoginDenied { .. } => {}
            };
        } else {
            error!("unwrapped packet from {peer_id:?}: {packet:?}");
        }
    }
}

/// Reads and handles server broadcast request events
pub(crate) fn broadcast(
    mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
    mut event_reader: EventReader<SilkBroadcastEvent>,
) {
    trace!("Trace 5: Broadcasting {} events", event_reader.len());
    // Silk broadcasts
    for broadcast in event_reader.iter() {
        match broadcast {
            // Unreliable operations
            SilkBroadcastEvent::UnreliableSendAllExcept((peer, packet)) => {
                let peers: Vec<PeerId> =
                    socket.connected_peers().filter(|p| p != peer).collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .send(packet.into_packet(), peer)
                })
            }
            SilkBroadcastEvent::UnreliableSendAll(packet) => {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .send(packet.into_packet(), peer)
                })
            }
            SilkBroadcastEvent::UnreliableSend((peer, packet)) => socket
                .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                .send(packet.into_packet(), *peer),

            // Reliable operations
            SilkBroadcastEvent::ReliableSendAllExcept((peer, packet)) => {
                let peers: Vec<PeerId> =
                    socket.connected_peers().filter(|p| p != peer).collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .send(packet.into_packet(), peer)
                })
            }
            SilkBroadcastEvent::ReliableSendAll(packet) => {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .send(packet.into_packet(), peer)
                })
            }
            SilkBroadcastEvent::ReliableSend((peer, packet)) => socket
                .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                .send(packet.into_packet(), *peer),
        }
    }
}
