use crate::{system_params::NetworkReader, ServerState};
use bevy::prelude::*;
use silk_common::{
    bevy_matchbox::{
        matchbox_socket::PeerState, prelude::MultipleChannels, MatchboxSocket,
        OpenSocketExt,
    },
    events::SilkServerEvent,
    packets::auth::SilkLoginRequestPayload,
    SilkSocket,
};

/// Initialize the socket
pub fn init_socket(mut commands: Commands, state: Res<ServerState>) {
    debug!("address: {:?}", state.addr);

    // Create matchbox socket
    let silk_socket = SilkSocket::new(state.addr.clone());
    commands.open_socket(silk_socket.builder());
}

/// Translates socket events into Bevy events
pub fn server_socket_reader(
    mut state: ResMut<ServerState>,
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
            PeerState::Connected => {
                // Authentication happens in another system! Do nothing.
            }
            PeerState::Disconnected => {
                event_wtr.send(SilkServerEvent::ClientLeft(peer));
            }
        }
    }
}

// Translate login requests to bevy server events
pub fn on_login(
    mut login_read: NetworkReader<SilkLoginRequestPayload>,
    mut event_wtr: EventWriter<SilkServerEvent>,
) {
    for (peer_id, payload) in login_read.iter() {
        match payload {
            SilkLoginRequestPayload::RegisteredUser {
                access_token,
                character,
            } => event_wtr.send(SilkServerEvent::LoginRequest {
                peer_id: *peer_id,
                access_token: access_token.clone(),
                character: character.clone(),
            }),
            SilkLoginRequestPayload::Guest { username } => {
                event_wtr.send(SilkServerEvent::GuestLoginRequest {
                    peer_id: *peer_id,
                    username: username.to_owned(),
                })
            }
        }
    }
}