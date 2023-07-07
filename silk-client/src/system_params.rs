use crate::router::{IncomingMessages, OutgoingMessages};
use bevy::{ecs::system::SystemParam, prelude::*};
use silk_net::Payload;

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Payload> {
    incoming: Res<'w, IncomingMessages<M>>,
}

impl<'w, M: Payload> NetworkReader<'w, M> {
    pub fn iter(&mut self) -> std::slice::Iter<'_, M> {
        self.incoming.messages.iter()
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, M: Payload> {
    outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Payload> NetworkWriter<'w, M> {
    pub fn reliable_to_host(&mut self, message: M) {
        self.outgoing.reliable_to_host.push(message);
    }

    pub fn unreliable_to_host(&mut self, message: M) {
        self.outgoing.unreliable_to_host.push(message);
    }
}
