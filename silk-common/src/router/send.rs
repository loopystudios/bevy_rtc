use bevy::prelude::*;
use bevy_matchbox::{
    prelude::{MultipleChannels, PeerId},
    MatchboxSocket,
};

use crate::SilkSocket;

use super::message::Message;
#[derive(Default, Debug, Resource)]
pub struct OutgoingMessages<M: Message> {
    pub reliable_all: Vec<M>,
    pub reliable: Vec<(PeerId, M)>,
}

impl<M: Message> OutgoingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general, this should be
    /// called once per frame/update.
    pub fn update(&mut self) {
        self.reliable_all.clear();
        self.reliable.clear();
    }

    /// A system that calls [`Events::update`] once per frame.
    pub fn update_system(mut query: ResMut<Self>) {
        query.update();
    }

    pub fn write_system(
        queue: ResMut<Self>,
        mut socket: ResMut<MatchboxSocket<MultipleChannels>>,
    ) {
        for message in queue.reliable_all.iter() {
            let peers: Vec<PeerId> = socket.connected_peers().collect();
            peers.into_iter().for_each(|peer| {
                socket
                    .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                    .send(message.to_packet(), peer)
            })
        }
        for (peer, message) in queue.reliable.iter() {
            socket
                .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                .send(message.to_packet(), *peer)
        }
    }
}
