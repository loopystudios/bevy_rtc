use crate::router::{IncomingMessages, OutgoingMessages};
use bevy::{ecs::system::SystemParam, prelude::*};
use silk_net::Message;

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Message> {
    incoming: Res<'w, IncomingMessages<M>>,
}

impl<'w, M: Message> NetworkReader<'w, M> {
    pub fn iter(&mut self) -> std::slice::Iter<'_, M> {
        self.incoming.messages.iter()
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, M: Message> {
    outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Message> NetworkWriter<'w, M> {
    pub fn reliable_to_host(&mut self, message: M) {
        self.outgoing.reliable_to_host.push(message);
    }

    pub fn unreliable_to_host(&mut self, message: M) {
        self.outgoing.unreliable_to_host.push(message);
    }
}
