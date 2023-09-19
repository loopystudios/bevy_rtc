mod receive;
mod send;

use bevy::prelude::*;
pub use receive::IncomingMessages;
pub use send::OutgoingMessages;
use silk_common::{
    bevy_matchbox::{prelude::MultipleChannels, MatchboxSocket},
    schedule::SilkSchedule,
    sets::SilkSet,
    socket::common_socket_reader,
};
pub use silk_net::Payload;

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
            panic!("client already contains resource: {}", M::reflect_name());
        }
        self.insert_resource(IncomingMessages::<M> { messages: vec![] })
            .add_systems(
                SilkSchedule,
                IncomingMessages::<M>::flush.in_set(SilkSet::Flush),
            )
            .add_systems(
                SilkSchedule,
                IncomingMessages::<M>::read_system
                    .before(SilkSet::NetworkRead)
                    .after(common_socket_reader)
                    .run_if(
                        resource_exists::<MatchboxSocket<MultipleChannels>>(),
                    ),
            )
            .insert_resource(OutgoingMessages::<M> {
                reliable_to_host: vec![],
                unreliable_to_host: vec![],
            })
            .add_systems(
                SilkSchedule,
                OutgoingMessages::<M>::write_system
                    .after(SilkSet::NetworkWrite),
            );
        self
    }
}
