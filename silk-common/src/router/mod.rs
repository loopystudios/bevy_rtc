use bevy::{ecs::system::SystemParam, prelude::*};

mod message;
mod receive;
mod send;

use bevy_matchbox::prelude::PeerId;
pub use message::Message;
pub use receive::IncomingMessages;

use crate::{schedule::SilkSchedule, SilkStage};

use self::send::OutgoingMessages;

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
    pub fn send_reliable_all(&mut self, message: &M) {
        self.outgoing.reliable_all.push(message.clone());
    }

    pub fn send_reliable(&mut self, peer: PeerId, message: &M) {
        self.outgoing.reliable.push((peer, message.clone()));
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
                    IncomingMessages::<T>::update_system
                        .in_base_set(CoreSet::First),
                )
                .add_system(
                    IncomingMessages::<T>::read_system
                        .in_base_set(SilkStage::ReadIn)
                        .in_schedule(SilkSchedule),
                );
        }
        if !self.world.contains_resource::<OutgoingMessages<T>>() {
            self.init_resource::<OutgoingMessages<T>>()
                .add_system(
                    OutgoingMessages::<T>::update_system
                        .in_base_set(CoreSet::Last),
                )
                .add_system(
                    OutgoingMessages::<T>::write_system
                        .in_base_set(SilkStage::WriteOut)
                        .in_schedule(SilkSchedule),
                );
        }
        self
    }
}
