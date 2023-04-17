mod receive;
mod send;

use bevy::prelude::*;
use silk_common::{
    schedule::SilkSchedule, socket::common_socket_reader, stage::SilkStage,
};
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
                        .before(SilkStage::Events)
                        .after(common_socket_reader)
                        .in_schedule(SilkSchedule),
                )
                .add_system(
                    IncomingMessages::<T>::flush
                        .before(common_socket_reader)
                        .in_schedule(SilkSchedule),
                );
        }
        if !self.world.contains_resource::<OutgoingMessages<T>>() {
            self.insert_resource(OutgoingMessages::<T> {
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
