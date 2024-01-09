use super::{
    events::{ConnectionRequest, SilkClientEvent},
    state::{SilkConnectionState, SilkState},
};
use crate::socket::{SilkSocket, SilkSocketPlurality};
use bevy::prelude::*;
use bevy_matchbox::{
    matchbox_socket::{self, WebRtcSocket},
    prelude::*,
};

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
    mut state: ResMut<SilkState>,
) {
    commands.close_socket::<SilkSocketPlurality>();
    *state = SilkState {
        // Keep for reconnecting
        addr: state.addr.clone(),
        host_id: None,
        id: None,
    };
}

/// Reads and handles connection request events
pub(crate) fn connection_request_handler(
    mut cxn_event_reader: EventReader<ConnectionRequest>,
    mut state: ResMut<SilkState>,
    mut next_connection_state: ResMut<NextState<SilkConnectionState>>,
    current_connection_state: Res<State<SilkConnectionState>>,
    mut event_wtr: EventWriter<SilkClientEvent>,
) {
    match cxn_event_reader.read().next() {
        Some(ConnectionRequest::Connect { addr }) => {
            if let SilkConnectionState::Disconnected =
                current_connection_state.get()
            {
                debug!(
                    previous = format!("{current_connection_state:?}"),
                    "set state: connecting"
                );
                state.addr.replace(addr.to_owned());
                next_connection_state.set(SilkConnectionState::Establishing);
            }
        }
        Some(ConnectionRequest::Disconnect) => {
            debug!(
                previous = format!("{current_connection_state:?}"),
                "set state: disconnected"
            );
            next_connection_state.set(SilkConnectionState::Disconnected);
            event_wtr.send(SilkClientEvent::DisconnectedFromHost {
                reason: Some("Client requested to disconnect".to_string()),
            });
        }
        None => {}
    }
}

/// Translates socket updates into bevy events
pub(crate) fn client_event_writer(
    mut state: ResMut<SilkState>,
    mut socket: ResMut<SilkSocket>,
    mut event_wtr: EventWriter<SilkClientEvent>,
    mut next_connection_state: ResMut<NextState<SilkConnectionState>>,
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
                        next_connection_state
                            .set(SilkConnectionState::Connected);
                        event_wtr.send(SilkClientEvent::ConnectedToHost(id));
                    }
                    matchbox_socket::PeerState::Disconnected => {
                        next_connection_state
                            .set(SilkConnectionState::Disconnected);
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
        next_connection_state.set(SilkConnectionState::Disconnected);
        event_wtr.send(SilkClientEvent::DisconnectedFromHost {
            reason: Some("Connection closed".to_string()),
        });
    }
}
