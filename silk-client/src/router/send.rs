use bevy::prelude::*;
use silk_common::bevy_matchbox::{prelude::MultipleChannels, MatchboxSocket};

use silk_common::SilkSocket;
use silk_net::Message;

use crate::state::ClientState;

#[derive(Default, Debug, Resource)]
pub struct OutgoingMessages<M: Message> {
    pub reliable_to_host: Vec<M>,
    pub unreliable_to_host: Vec<M>,
}

impl<M: Message> OutgoingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general,
    /// this should be called once per frame/update.
    pub fn update(&mut self) {
        self.reliable_to_host.clear();
        self.unreliable_to_host.clear();
    }

    pub(crate) fn write_system(
        mut queue: ResMut<Self>,
        mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
        state: Res<ClientState>,
    ) {
        if let Some(socket) = socket.as_mut() {
            if let Some(host) = state.host_id {
                // Client is sending
                for message in queue.reliable_to_host.iter() {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .send(message.to_packet(), host)
                }
                for message in queue.unreliable_to_host.iter() {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .send(message.to_packet(), host)
                }
            }
            queue.update();
        }
    }
}
