use bevy::{ecs::system::SystemParam, prelude::*};

mod message;
mod receive;

pub use message::Message;
pub use receive::IncomingMessages;

use crate::{schedule::SilkSchedule, SilkStage};

#[derive(SystemParam, Debug)]
pub struct NetworkReader<'w, M: Message> {
    incoming: Res<'w, IncomingMessages<M>>,
}

impl<'w, M: Message> NetworkReader<'w, M> {
    pub fn iter(&mut self) -> std::slice::Iter<'_, M> {
        self.incoming.messages.iter()
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
        self
    }
}
