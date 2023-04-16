use bevy::{ecs::system::SystemParam, prelude::*};

mod message;
mod receive;
mod send;

use bevy_matchbox::prelude::PeerId;
pub use message::Message;
pub use receive::IncomingMessages;

use crate::{schedule::SilkSchedule, socket::socket_reader, SilkStage};

use self::send::OutgoingMessages;

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Message> {
    incoming: Res<'w, IncomingMessages<M>>,
}

impl<'w, M: Message> NetworkReader<'w, M> {
    pub fn iter(&mut self) -> std::slice::Iter<'_, (PeerId, M)> {
        self.incoming.messages.iter()
    }
}

#[derive(SystemParam, Debug)]
pub struct NetworkWriter<'w, M: Message> {
    outgoing: ResMut<'w, OutgoingMessages<M>>,
}

impl<'w, M: Message> NetworkWriter<'w, M> {
    pub fn reliable_to_all(&mut self, message: &M) {
        self.outgoing.reliable_to_all.push(message.clone());
    }

    pub fn reliable_to_all_except(&mut self, peer: PeerId, message: &M) {
        self.outgoing
            .reliable_to_all_except
            .push((peer, message.clone()));
    }

    pub fn reliable_to_peer(&mut self, peer: PeerId, message: &M) {
        self.outgoing.reliable_to_peer.push((peer, message.clone()));
    }

    pub fn unreliable_to_peer(&mut self, peer: PeerId, message: &M) {
        self.outgoing
            .unreliable_to_peer
            .push((peer, message.clone()));
    }

    pub fn reliable_to_host(&mut self, message: &M) {
        self.outgoing.reliable_to_host.push(message.clone());
    }

    pub fn unreliable_to_host(&mut self, message: &M) {
        self.outgoing.unreliable_to_host.push(message.clone());
    }
}

pub trait AddNetworkMessage {
    fn add_network_message<T: Message>(&mut self) -> &mut Self;
}

impl AddNetworkMessage for App {
    fn add_network_message<T>(&mut self) -> &mut Self
    where
        T: Message,
    {
        if !self.world.contains_resource::<IncomingMessages<T>>() {
            self.init_resource::<IncomingMessages<T>>()
                .add_system(
                    IncomingMessages::<T>::read_system
                        .before(SilkStage::ReadIn)
                        .after(socket_reader)
                        .in_schedule(SilkSchedule),
                )
                .add_system(
                    IncomingMessages::<T>::update_system
                        .in_base_set(CoreSet::Last),
                );
        }
        if !self.world.contains_resource::<OutgoingMessages<T>>() {
            self.init_resource::<OutgoingMessages<T>>().add_system(
                OutgoingMessages::<T>::write_system
                    .after(SilkStage::WriteOut)
                    .in_schedule(SilkSchedule),
            );
        }
        self
    }
}
