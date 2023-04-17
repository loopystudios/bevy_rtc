use crate::{events::SocketRecvEvent, SilkSocket};
use bevy::prelude::*;
use bevy_matchbox::{prelude::MultipleChannels, MatchboxSocket};

pub fn common_socket_reader(
    mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
    mut event_wtr: EventWriter<SocketRecvEvent>,
) {
    if let Some(socket) = socket.as_mut() {
        // Collect Unreliable, Reliable messages
        let messages = socket
            .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
            .receive()
            .into_iter()
            .chain(
                socket
                    .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                    .receive(),
            )
            .map(SocketRecvEvent)
            .collect::<Vec<_>>();
        event_wtr.send_batch(messages);
    }
}
