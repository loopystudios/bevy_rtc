use crate::{events::SocketRecvEvent, SilkSocket};
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

pub fn socket_reader(
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
        trace!("received {} messages", messages.len());
        event_wtr.send_batch(messages);
    }
}
