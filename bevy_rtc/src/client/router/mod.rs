mod receive;
mod send;

use crate::{
    protocol::Protocol,
    socket::{common_socket_reader, RtcSocket},
};
use bevy::prelude::*;
use std::collections::VecDeque;

pub use receive::IncomingMessages;
pub use send::OutgoingMessages;

pub trait AddClientProtocolExt {
    /// Register a protocol that is only written, never read.
    fn add_client_wo_protocol<M: Protocol>(&mut self) -> &mut Self;
    /// Register a protocol that is only read, never written. Allocate a bounded
    /// buffer per peer for receiving.
    fn add_client_ro_protocol<M: Protocol>(&mut self, bound: usize) -> &mut Self;
    /// Register a protocol that is only read, never written. Use a growable buffer
    /// for reading.
    fn add_client_ro_unbounded_protocol<M: Protocol>(&mut self) -> &mut Self;
    /// Register a protocol for reading and writing. Allocate a bounded buffer
    /// per peer for reading.
    fn add_client_rw_protocol<M: Protocol>(&mut self, bound: usize) -> &mut Self;
    /// Register a protocol for sending and receiving. Use a growable buffer
    /// for reading.
    fn add_client_rw_unbounded_protocol<M: Protocol>(&mut self) -> &mut Self;
}

impl AddClientProtocolExt for App {
    fn add_client_wo_protocol<M: Protocol>(&mut self) -> &mut Self {
        if self.world.contains_resource::<OutgoingMessages<M>>() {
            panic!("client already contains resource: {}", M::reflect_name());
        }
        self.insert_resource(OutgoingMessages::<M> {
            reliable_to_host: vec![],
            unreliable_to_host: vec![],
        })
        .add_systems(
            Last,
            OutgoingMessages::<M>::send_payloads.run_if(resource_exists::<RtcSocket>),
        );
        self
    }

    fn add_client_ro_protocol<M: Protocol>(&mut self, bound: usize) -> &mut Self {
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

    fn add_client_ro_unbounded_protocol<M: Protocol>(&mut self) -> &mut Self {
        self.add_client_ro_protocol::<M>(usize::MAX)
    }

    fn add_client_rw_protocol<M>(&mut self, bound: usize) -> &mut Self
    where
        M: Protocol,
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
            OutgoingMessages::<M>::send_payloads.run_if(resource_exists::<RtcSocket>),
        );
        self
    }

    fn add_client_rw_unbounded_protocol<M: Protocol>(&mut self) -> &mut Self {
        self.add_client_rw_protocol::<M>(usize::MAX)
    }
}
