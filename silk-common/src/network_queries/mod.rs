use bevy::prelude::*;

mod message;
mod receive;

pub use message::Message;
pub use receive::{NetworkQuery, RecvMessages};

use crate::{schedule::SilkSchedule, SilkStage};

pub trait AddNetworkQuery {
    fn add_network_query<T: Message>(&mut self) -> &mut Self;
}

impl AddNetworkQuery for App {
    fn add_network_query<T>(&mut self) -> &mut Self
    where
        T: Message,
    {
        if !self.world.contains_resource::<RecvMessages<T>>() {
            self.init_resource::<RecvMessages<T>>()
                .add_system(
                    RecvMessages::<T>::update_system
                        .in_base_set(CoreSet::First),
                )
                .add_system(
                    RecvMessages::<T>::read_system
                        .in_base_set(SilkStage::ReadIn)
                        .in_schedule(SilkSchedule),
                );
        }
        self
    }
}
