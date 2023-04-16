mod receive;
mod send;
mod system_params;

use self::send::OutgoingMessages;
use crate::{schedule::SilkSchedule, socket::socket_reader, SilkStage};
use bevy::prelude::*;
use silk_net::Message;

pub use receive::IncomingMessages;
pub use system_params::{NetworkReader, NetworkWriter};

pub trait AddNetworkMessageExt {
    fn add_network_message<T: Message>(&mut self) -> &mut Self;
}

impl AddNetworkMessageExt for App {
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
