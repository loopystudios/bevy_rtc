use bevy::{ecs::system::SystemParam, prelude::*};

use bevy_matchbox::prelude::PeerId;

use crate::router::{IncomingMessages, OutgoingMessages};
use silk_net::Payload;

use silk_common::bevy_matchbox;

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Payload> {
    incoming: Res<'w, IncomingMessages<M>>,
}

impl<'w, M: Payload> NetworkReader<'w, M> {
    pub fn iter(&mut self) -> std::slice::Iter<'_, (PeerId, M)> {
        self.incoming.messages.iter()
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, M: Payload> {
    outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Payload> NetworkWriter<'w, M> {
    pub fn reliable_to_all(&mut self, message: M) {
        self.outgoing.reliable_to_all.push(message);
    }

    pub fn unreliable_to_all(&mut self, message: M) {
        self.outgoing.unreliable_to_all.push(message);
    }

    pub fn reliable_to_all_except(&mut self, peer: PeerId, message: M) {
        self.outgoing.reliable_to_all_except.push((peer, message));
    }

    pub fn unreliable_to_all_except(&mut self, peer: PeerId, message: M) {
        self.outgoing.unreliable_to_all_except.push((peer, message));
    }

    pub fn reliable_to_peer(&mut self, peer: PeerId, message: M) {
        self.outgoing.reliable_to_peer.push((peer, message));
    }

    pub fn unreliable_to_peer(&mut self, peer: PeerId, message: M) {
        self.outgoing.unreliable_to_peer.push((peer, message));
    }
}
