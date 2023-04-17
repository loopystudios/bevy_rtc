use bevy::{ecs::system::SystemParam, prelude::*};
use silk_net::Message;

use crate::router::{IncomingMessages, OutgoingMessages};

#[derive(SystemParam, Debug)]
pub struct ClientRecv<'w, M: Message> {
    incoming: Res<'w, IncomingMessages<M>>,
}

impl<'w, M: Message> ClientRecv<'w, M> {
    pub fn iter(&mut self) -> std::slice::Iter<'_, M> {
        self.incoming.messages.iter()
    }
}

#[derive(SystemParam, Debug)]
pub struct ClientSend<'w, M: Message> {
    outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Message> ClientSend<'w, M> {
    pub fn reliable_to_host(&mut self, message: M) {
        self.outgoing.reliable_to_host.push(message);
    }

    pub fn unreliable_to_host(&mut self, message: M) {
        self.outgoing.unreliable_to_host.push(message);
    }
}
