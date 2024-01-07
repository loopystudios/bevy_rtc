mod receive;
mod send;

use crate::{
    protocol::Payload,
    schedule::{SilkSchedule, SilkSet},
    socket::{common_socket_reader, SilkSocket},
};
use bevy::prelude::*;

pub use receive::IncomingMessages;
pub use send::OutgoingMessages;

pub trait AddNetworkMessageExt {
    fn add_network_message<M: Payload>(&mut self) -> &mut Self;
}

impl AddNetworkMessageExt for App {
    fn add_network_message<M>(&mut self) -> &mut Self
    where
        M: Payload,
    {
        if self.world.contains_resource::<IncomingMessages<M>>()
            || self.world.contains_resource::<OutgoingMessages<M>>()
        {
            panic!("server already contains resource: {}", M::reflect_name());
        }
        self.insert_resource(IncomingMessages::<M> { messages: vec![] })
            .insert_resource(OutgoingMessages::<M> {
                reliable_to_all: vec![],
                unreliable_to_all: vec![],
                reliable_to_all_except: vec![],
                unreliable_to_all_except: vec![],
                reliable_to_peer: vec![],
                unreliable_to_peer: vec![],
            })
            .add_systems(
                SilkSchedule,
                (
                    IncomingMessages::<M>::flush,
                    IncomingMessages::<M>::receive_payloads,
                )
                    .chain()
                    .before(SilkSet::PreUpdate)
                    .after(common_socket_reader),
            )
            .add_systems(
                SilkSchedule,
                OutgoingMessages::<M>::send_payloads
                    .after(SilkSet::PostUpdate)
                    .run_if(resource_exists::<SilkSocket>()),
            );

        self
    }
}
