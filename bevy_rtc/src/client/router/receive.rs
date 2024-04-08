use crate::{events::SocketRecvEvent, protocol::Protocol};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Protocol> {
    pub bound: usize,
    pub messages: VecDeque<M>,
}

impl<M: Protocol> IncomingMessages<M> {
    pub fn receive_payloads(mut incoming: ResMut<Self>, mut events: EventReader<SocketRecvEvent>) {
        let bound = incoming.bound;
        let packets: Vec<_> = events
            .read()
            .map(|&SocketRecvEvent((_peer_id, ref packet))| packet)
            .filter_map(M::from_packet)
            .enumerate()
            .take_while(|(read, _)| *read <= bound)
            .map(|(_, packet)| packet)
            .collect();
        trace!("Read {} {} packets", packets.len(), M::reflect_name());
        for packet in packets.into_iter() {
            incoming.messages.push_back(packet);
        }
        if incoming.messages.len() > bound {
            warn!(
                "The `{}` protocol is overflowing its bounded buffer ({bound}) and dropping packets! Is it being read?",
                M::reflect_name()
            );
            while incoming.messages.len() > bound {
                incoming.messages.pop_front();
            }
        }
    }
}
