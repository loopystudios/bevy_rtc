use std::collections::VecDeque;

use crate::{events::SocketRecvEvent, protocol::Payload};
use bevy::{prelude::*, utils::HashMap};
use bevy_matchbox::prelude::PeerId;

#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Payload> {
    pub bound: usize,
    pub messages: HashMap<PeerId, VecDeque<M>>,
}

impl<M: Payload> IncomingMessages<M> {
    pub fn receive_payloads(
        mut incoming: ResMut<Self>,
        mut events: EventReader<SocketRecvEvent>,
    ) {
        let mut read = 0;
        let bound = incoming.bound;
        for SocketRecvEvent((peer_id, packet)) in events.read() {
            if let Some(message) = M::from_packet(packet) {
                // Get or insert the VecDeque for the peer_id
                let messages_for_peer = incoming
                    .messages
                    .entry(*peer_id)
                    .or_insert_with(VecDeque::new);
                // Insert the new message
                messages_for_peer.push_back(message);
                // Ensure only the last BOUND messages are kept
                while messages_for_peer.len() > bound {
                    messages_for_peer.pop_front();
                    warn!(
                        "The `{}` protocol is overflowing its bounded buffer ({bound}) and dropping packets! The payloads may not being read fast enough, or {peer_id} is exceeding rate!",
                        M::reflect_name()
                    );
                }
                read += 1;
            }
        }
        if read > 0 {
            trace!("received {} {} packets", read, M::reflect_name());
        }
    }
}
