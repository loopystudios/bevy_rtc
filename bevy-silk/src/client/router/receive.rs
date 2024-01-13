use crate::{events::SocketRecvEvent, protocol::Payload};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Payload> {
    pub bound: usize,
    pub messages: VecDeque<M>,
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
                // Insert the new message
                incoming.messages.push_back(message);
                // Ensure only the last BOUND messages are kept
                while incoming.messages.len() > bound {
                    incoming.messages.pop_front();
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
