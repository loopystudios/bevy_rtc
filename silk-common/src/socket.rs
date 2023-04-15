use crate::{events::SocketRecvEvent, SilkSocket, SilkSocketEvent};
use bevy::prelude::*;
use bevy_matchbox::{
    prelude::{MultipleChannels, PeerId},
    MatchboxSocket,
};

#[derive(Resource, Default)]
pub struct SocketState {
    /// The ID of the host if this instance is a client
    pub host: Option<PeerId>,
}

pub fn handle_socket_events(
    mut state: ResMut<SocketState>,
    mut events: EventReader<SilkSocketEvent>,
) {
    for event in events.iter() {
        match event {
            SilkSocketEvent::ConnectedToHost(id) => {
                state.host.replace(*id);
            }
            SilkSocketEvent::DisconnectedFromHost => {
                state.host.take();
            }
            _ => {}
        }
    }
}

pub fn socket_reader(
    mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
    mut event_wtr: EventWriter<SocketRecvEvent>,
) {
    if let Some(socket) = socket.as_mut() {
        // Collect Unreliable, Reliable messages
        let reliable_msgs =
            socket.channel(SilkSocket::RELIABLE_CHANNEL_INDEX).receive();
        let unreliable_msgs = socket
            .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
            .receive();

        event_wtr.send_batch(
            reliable_msgs
                .into_iter()
                .chain(unreliable_msgs)
                .map(SocketRecvEvent),
        );
    }
}
