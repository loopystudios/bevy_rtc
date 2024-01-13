mod receive;
mod send;

use crate::{
    protocol::Payload,
    socket::{common_socket_reader, RtcSocket},
};
use bevy::prelude::*;
use std::collections::VecDeque;

pub use receive::IncomingMessages;
pub use send::OutgoingMessages;

pub trait AddProtocolExt {
    /// Register a protocol, dually allocating a sized buffer for
    /// payloads received, per peer.
    fn add_bounded_protocol<M: Payload>(&mut self, bound: usize) -> &mut Self;
    /// Register a protocol, with a growable buffer.
    fn add_unbounded_protocol<M: Payload>(&mut self) -> &mut Self;
}

impl AddProtocolExt for App {
    fn add_unbounded_protocol<M: Payload>(&mut self) -> &mut Self {
        self.add_bounded_protocol::<M>(usize::MAX)
    }

    fn add_bounded_protocol<M>(&mut self, bound: usize) -> &mut Self
    where
        M: Payload,
    {
        if self.world.contains_resource::<IncomingMessages<M>>()
            || self.world.contains_resource::<OutgoingMessages<M>>()
        {
            panic!("client already contains resource: {}", M::reflect_name());
        }
        self.insert_resource(IncomingMessages::<M> {
            bound,
            messages: VecDeque::new(),
        })
        .insert_resource(OutgoingMessages::<M> {
            reliable_to_host: vec![],
            unreliable_to_host: vec![],
        })
        .add_systems(
            First,
            IncomingMessages::<M>::receive_payloads
                .after(common_socket_reader)
                .run_if(resource_exists::<RtcSocket>()),
        )
        .add_systems(
            Last,
            OutgoingMessages::<M>::send_payloads
                .run_if(resource_exists::<RtcSocket>()),
        );
        self
    }
}
