use bevy::prelude::*;
use silk_common::{
    bevy_matchbox::{
        prelude::{MultipleChannels, PeerId},
        MatchboxSocket,
    },
    SilkSocket,
};
use silk_net::Message;

#[derive(Default, Debug, Resource)]
pub struct OutgoingMessages<M: Message> {
    pub reliable_to_all: Vec<M>,
    pub unreliable_to_all: Vec<M>,
    pub reliable_to_all_except: Vec<(PeerId, M)>,
    pub unreliable_to_all_except: Vec<(PeerId, M)>,
    pub reliable_to_peer: Vec<(PeerId, M)>,
    pub unreliable_to_peer: Vec<(PeerId, M)>,
}

impl<M: Message> OutgoingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general,
    /// this should be called once per frame/update.
    pub fn flush(&mut self) {
        self.reliable_to_all.clear();
        self.unreliable_to_all.clear();
        self.reliable_to_all_except.clear();
        self.unreliable_to_all_except.clear();
        self.reliable_to_peer.clear();
        self.unreliable_to_peer.clear();
    }

    pub(crate) fn write_system(
        mut queue: ResMut<Self>,
        mut socket: Option<ResMut<MatchboxSocket<MultipleChannels>>>,
    ) {
        if let Some(socket) = socket.as_mut() {
            trace!(
                "sending {} {} packets",
                queue.reliable_to_all.len()
                    + queue.unreliable_to_all.len()
                    + queue.reliable_to_all_except.len()
                    + queue.unreliable_to_all_except.len()
                    + queue.reliable_to_peer.len()
                    + queue.unreliable_to_peer.len(),
                M::reflect_name()
            );
            // Server is sending
            for message in queue.reliable_to_all.iter() {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .send(message.to_packet(), peer)
                })
            }
            for message in queue.unreliable_to_all.iter() {
                let peers: Vec<PeerId> = socket.connected_peers().collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .send(message.to_packet(), peer)
                })
            }
            for (peer, message) in queue.reliable_to_all_except.iter() {
                let peers: Vec<PeerId> =
                    socket.connected_peers().filter(|p| p != peer).collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                        .send(message.to_packet(), peer)
                });
            }
            for (peer, message) in queue.unreliable_to_all_except.iter() {
                let peers: Vec<PeerId> =
                    socket.connected_peers().filter(|p| p != peer).collect();
                peers.into_iter().for_each(|peer| {
                    socket
                        .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                        .send(message.to_packet(), peer)
                });
            }
            for (peer, message) in queue.reliable_to_peer.iter() {
                socket
                    .channel(SilkSocket::RELIABLE_CHANNEL_INDEX)
                    .send(message.to_packet(), *peer)
            }
            for (peer, message) in queue.unreliable_to_peer.iter() {
                socket
                    .channel(SilkSocket::UNRELIABLE_CHANNEL_INDEX)
                    .send(message.to_packet(), *peer)
            }

            queue.flush();
        }
    }
}
