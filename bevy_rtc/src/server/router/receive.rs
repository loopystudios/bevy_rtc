use crate::{events::SocketRecvEvent, protocol::Protocol, transport_encoding::TransportEncoding};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_matchbox::prelude::PeerId;
use std::collections::VecDeque;

#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Protocol> {
    pub bound: usize,
    pub messages: HashMap<PeerId, VecDeque<M>>,
}

impl<M: Protocol> IncomingMessages<M> {
    pub fn receive_payloads(
        mut incoming: ResMut<Self>,
        mut events: EventReader<SocketRecvEvent>,
        encoding: Res<TransportEncoding>,
    ) {
        let bound = incoming.bound;
        let packets: HashMap<PeerId, Vec<M>> = events.read().fold(
            HashMap::new(),
            |mut acc, &SocketRecvEvent((peer_id, ref packet))| {
                let buf = acc.entry(peer_id).or_insert(vec![]);
                if buf.len() >= bound {
                    return acc;
                }
                if let Some(packet) = M::from_packet(packet, &encoding) {
                    buf.push(packet);
                }
                acc
            },
        );
        for (peer_id, payloads) in packets {
            // Get or insert the VecDeque for the peer_id
            let messages_for_peer = incoming
                .messages
                .entry(peer_id)
                .or_insert_with(VecDeque::new);
            for payload in payloads.into_iter() {
                messages_for_peer.push_back(payload);
            }
            if messages_for_peer.len() > bound {
                warn!(
                    "The `{}` protocol is overflowing its bounded buffer ({bound}) and dropping packets! Is it being read?",
                    M::reflect_name()
                );
                while messages_for_peer.len() > bound {
                    messages_for_peer.pop_front();
                }
            }
        }
    }
}
