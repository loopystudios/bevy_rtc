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
    /// Register a protocol that is only sent, never read. Hence, allocate no
    /// buffer and do not run systems for receiving.
    fn add_sendonly_protocol<M: Payload>(&mut self) -> &mut Self;
    /// Register a protocol that is only read, never sent. Allocate a bounded
    /// buffer per peer for receiving, and do not run systems for sending.
    fn add_readonly_bounded_protocol<M: Payload>(
        &mut self,
        bound: usize,
    ) -> &mut Self;
    /// Register a protocol that is only read, never sent. Use a growable buffer
    /// for receiving, and do not run systems for sending.
    fn add_readonly_unbounded_protocol<M: Payload>(&mut self) -> &mut Self;
    /// Register a protocol for sending and receiving. Allocate a bounded buffer
    /// per peer for receiving.
    fn add_bounded_protocol<M: Payload>(&mut self, bound: usize) -> &mut Self;
    /// Register a protocol for sending and receiving. Use a growable buffer
    /// for receiving.
    fn add_unbounded_protocol<M: Payload>(&mut self) -> &mut Self;
}

impl AddProtocolExt for App {
    fn add_sendonly_protocol<M: Payload>(&mut self) -> &mut Self {
        if self.world.contains_resource::<OutgoingMessages<M>>() {
            panic!("client already contains resource: {}", M::reflect_name());
        }
        self.insert_resource(OutgoingMessages::<M> {
            reliable_to_host: vec![],
            unreliable_to_host: vec![],
        })
        .add_systems(
            Last,
            OutgoingMessages::<M>::send_payloads
                .run_if(resource_exists::<RtcSocket>),
        );
        self
    }

    fn add_readonly_unbounded_protocol<M: Payload>(&mut self) -> &mut Self {
        self.add_readonly_bounded_protocol::<M>(usize::MAX)
    }

    fn add_readonly_bounded_protocol<M: Payload>(
        &mut self,
        bound: usize,
    ) -> &mut Self {
        if self.world.contains_resource::<IncomingMessages<M>>() {
            panic!("client already contains resource: {}", M::reflect_name());
        }
        self.insert_resource(IncomingMessages::<M> {
            bound,
            messages: VecDeque::new(),
        })
        .add_systems(
            First,
            IncomingMessages::<M>::receive_payloads
                .after(common_socket_reader)
                .run_if(resource_exists::<RtcSocket>),
        );
        self
    }

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
                .run_if(resource_exists::<RtcSocket>),
        )
        .add_systems(
            Last,
            OutgoingMessages::<M>::send_payloads
                .run_if(resource_exists::<RtcSocket>),
        );
        self
    }
}
