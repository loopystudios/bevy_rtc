use crate::SocketState;
use bevy::prelude::*;
use silk_common::SilkSocket;
use silk_common::{
    bevy_matchbox::{
        matchbox_socket::PeerState, prelude::MultipleChannels, MatchboxSocket,
        OpenSocketExt,
    },
    SilkSocketEvent,
};

/// Initialize the socket
pub fn init_socket(mut commands: Commands, state: Res<SocketState>) {
    debug!("address: {:?}", state.addr);

    // Create matchbox socket
    let silk_socket = SilkSocket::new(state.addr);
    commands.open_socket(silk_socket.builder());
}

/// Translates socket events into Bevy events
pub fn socket_reader(
    mut state: ResMut<SocketState>,
    mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
    mut event_wtr: EventWriter<SilkSocketEvent>,
) {
    // Id changed events
    if let Some(id) = socket.id() {
        if state.id.is_none() {
            state.id.replace(id);
            event_wtr.send(SilkSocketEvent::IdAssigned(id));
        }
    }

    // Check for peer updates
    for (peer, peer_state) in socket.update_peers() {
        match peer_state {
            PeerState::Connected => {
                event_wtr.send(SilkSocketEvent::ClientJoined(peer));
            }
            PeerState::Disconnected => {
                event_wtr.send(SilkSocketEvent::ClientLeft(peer));
            }
        }
    }
}
