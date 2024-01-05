use super::{
    events::{ConnectionRequest, SilkClientEvent},
    state::{ClientState, ConnectionState},
    system_params::{NetworkReader, NetworkWriter},
};
use crate::{
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    protocol::AuthenticationRequest,
};
use bevy::prelude::*;
use bevy_matchbox::{
    matchbox_socket::{self, WebRtcSocket},
    prelude::*,
};

/// Initialize the socket
pub(crate) fn init_socket(
    mut commands: Commands,
    socket_res: Res<ClientState>,
) {
    if let Some(addr) = &socket_res.addr {
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
    mut state: ResMut<ClientState>,
) {
    commands.close_socket::<MultipleChannels>();
    *state = ClientState::default();
}

/// Reads and handles connection request events
pub(crate) fn connection_event_reader(
    mut cxn_event_reader: EventReader<ConnectionRequest>,
    mut state: ResMut<ClientState>,
    mut next_connection_state: ResMut<NextState<ConnectionState>>,
    current_connection_state: Res<State<ConnectionState>>,
    mut event_wtr: EventWriter<SilkClientEvent>,
) {
    match cxn_event_reader.read().next() {
        Some(ConnectionRequest::Connect { addr, auth }) => {
            if let ConnectionState::Disconnected =
                current_connection_state.get()
            {
                debug!(
                    previous = format!("{current_connection_state:?}"),
                    "set state: connecting"
                );
                state.addr.replace(addr.to_owned());
                state.auth.replace(auth.to_owned());
                next_connection_state.set(ConnectionState::Establishing);
            }
        }
        Some(ConnectionRequest::Disconnect { reason }) => {
            debug!(
                previous = format!("{current_connection_state:?}"),
                "set state: disconnected"
            );
            next_connection_state.set(ConnectionState::Disconnected);
            event_wtr.send(SilkClientEvent::DisconnectedFromHost {
                reason: reason.to_owned(),
            });
        }
        None => {}
    }
}

/// Translates socket updates into bevy events
pub(crate) fn client_socket_reader(
    mut state: ResMut<ClientState>,
    mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
    mut event_wtr: EventWriter<SilkClientEvent>,
    mut login_send: NetworkWriter<SilkLoginRequestPayload>,
    mut next_connection_state: ResMut<NextState<ConnectionState>>,
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
                        let Some(auth) = state.auth.take() else {
                            panic!("no auth set")
                        };
                        match auth {
                            AuthenticationRequest::Registered {
                                access_token,
                                character,
                            } => login_send.reliable_to_host(
                                SilkLoginRequestPayload::RegisteredUser {
                                    access_token,
                                    character,
                                },
                            ),
                            AuthenticationRequest::Guest { username } => {
                                login_send.reliable_to_host(
                                    SilkLoginRequestPayload::Guest { username },
                                )
                            }
                        }
                    }
                    matchbox_socket::PeerState::Disconnected => {
                        next_connection_state
                            .set(ConnectionState::Disconnected);
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
        next_connection_state.set(ConnectionState::Disconnected);
        event_wtr.send(SilkClientEvent::DisconnectedFromHost {
            reason: Some("Connection failed".to_string()),
        });
    }
}

// Translate login to bevy client events
pub(crate) fn on_login_accepted(
    state: Res<ClientState>,
    mut next_connection_state: ResMut<NextState<ConnectionState>>,
    mut login_read: NetworkReader<SilkLoginResponsePayload>,
    mut event_wtr: EventWriter<SilkClientEvent>,
) {
    for payload in login_read.iter() {
        match payload {
            SilkLoginResponsePayload::Accepted { username } => {
                info!("authenticated user: {username}");
                next_connection_state.set(ConnectionState::Connected);
                event_wtr.send(SilkClientEvent::ConnectedToHost {
                    host: state.host_id.unwrap(),
                    username: username.to_string(),
                });
            }
            SilkLoginResponsePayload::Denied { reason } => {
                error!("login denied, reason: {reason:?}");
                next_connection_state.set(ConnectionState::Disconnected);
                event_wtr.send(SilkClientEvent::DisconnectedFromHost {
                    reason: reason.to_owned(),
                });
            }
        }
    }
}
