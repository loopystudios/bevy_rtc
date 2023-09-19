use crate::events::SocketRecvEvent;
use bevy::prelude::*;
use bevy_matchbox::{prelude::MultipleChannels, MatchboxSocket};

/// The index of the unreliable channel in the [`WebRtcSocket`].
pub const UNRELIABLE_CHANNEL_INDEX: usize = 0;
/// The index of the reliable channel in the [`WebRtcSocket`].
pub const RELIABLE_CHANNEL_INDEX: usize = 1;

pub fn common_socket_reader(
    mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
    mut event_wtr: EventWriter<SocketRecvEvent>,
) {
    let messages = socket
        .channel(RELIABLE_CHANNEL_INDEX)
        .receive()
        .into_iter()
        .chain(socket.channel(UNRELIABLE_CHANNEL_INDEX).receive())
        .map(SocketRecvEvent)
        .collect::<Vec<_>>();
    trace!("Received {} total messages", messages.len());

    event_wtr.send_batch(messages);
}
