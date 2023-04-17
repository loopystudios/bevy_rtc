mod receive;
mod send;

use bevy::prelude::*;
use silk_common::{
    schedule::SilkSchedule, socket::common_socket_reader, SilkStage,
};
pub use silk_net::Message;

pub use receive::IncomingMessages;
pub use send::OutgoingMessages;

pub trait AddNetworkMessageExt {
    fn add_network_message<M: Message>(&mut self) -> &mut Self;
}

impl AddNetworkMessageExt for App {
    fn add_network_message<M>(&mut self) -> &mut Self
    where
        M: Message,
    {
        if self.world.contains_resource::<IncomingMessages<M>>()
            || self.world.contains_resource::<OutgoingMessages<M>>()
        {
            panic!("server already contains resource: {}", M::reflect_name());
        }
        self.insert_resource(IncomingMessages::<M> { messages: vec![] })
            .add_system(
                IncomingMessages::<M>::read_system
                    .before(SilkStage::Events)
                    .after(common_socket_reader)
                    .in_schedule(SilkSchedule),
            )
            .add_system(
                IncomingMessages::<M>::flush
                    .before(common_socket_reader)
                    .in_schedule(SilkSchedule),
            )
            .insert_resource(OutgoingMessages::<M> {
                reliable_to_all: vec![],
                unreliable_to_all: vec![],
                reliable_to_all_except: vec![],
                unreliable_to_all_except: vec![],
                reliable_to_peer: vec![],
                unreliable_to_peer: vec![],
            })
            .add_system(
                OutgoingMessages::<M>::write_system
                    .after(SilkStage::WriteOut)
                    .in_schedule(SilkSchedule),
            );

        self
    }
}
