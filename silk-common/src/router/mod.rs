mod receive;
mod send;

use crate::{schedule::SilkSchedule, socket::socket_reader, SilkStage};
use bevy::prelude::*;
pub use silk_net::Message;

pub use receive::IncomingMessages;
pub use send::OutgoingMessages;

pub trait AddNetworkMessageExt {
    fn add_network_message<T: Message>(&mut self) -> &mut Self;
}

impl AddNetworkMessageExt for App {
    fn add_network_message<T>(&mut self) -> &mut Self
    where
        T: Message,
    {
        if !self.world.contains_resource::<IncomingMessages<T>>() {
            self.insert_resource(IncomingMessages::<T> { messages: vec![] })
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
            self.insert_resource(OutgoingMessages::<T> {
                reliable_to_all: vec![],
                unreliable_to_all: vec![],
                reliable_to_all_except: vec![],
                unreliable_to_all_except: vec![],
                reliable_to_peer: vec![],
                unreliable_to_peer: vec![],
                reliable_to_host: vec![],
                unreliable_to_host: vec![],
            })
            .add_system(
                OutgoingMessages::<T>::write_system
                    .after(SilkStage::WriteOut)
                    .in_schedule(SilkSchedule),
            );
        }
        self
    }
}
