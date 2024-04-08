use crate::{
    client::state::RtcClientState,
    protocol::Payload,
    socket::{RtcSocket, RELIABLE_CHANNEL_INDEX, UNRELIABLE_CHANNEL_INDEX},
};
use bevy::prelude::*;

#[derive(Default, Debug, Resource)]
pub struct OutgoingMessages<M: Payload> {
    pub reliable_to_host: Vec<M>,
    pub unreliable_to_host: Vec<M>,
}

impl<M: Payload> OutgoingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general,
    /// this should be called once per frame/update.
    pub fn flush(&mut self) {
        self.reliable_to_host.clear();
        self.unreliable_to_host.clear();
    }

    pub(crate) fn send_payloads(
        mut queue: ResMut<Self>,
        mut socket: ResMut<RtcSocket>,
        state: Res<RtcClientState>,
    ) {
        if let Some(host) = state.host_id {
            // Client is sending
            for message in queue.reliable_to_host.iter() {
                if socket
                    .channel_mut(RELIABLE_CHANNEL_INDEX)
                    .try_send(message.to_packet(), host)
                    .is_err()
                {
                    error!("failed to send reliable packet to {host}: {message:?}");
                }
            }
            if !queue.reliable_to_host.is_empty() {
                trace!(
                    "sent {} [R] {} packets",
                    queue.reliable_to_host.len(),
                    M::reflect_name()
                );
            }
            for message in queue.unreliable_to_host.iter() {
                if socket
                    .channel_mut(UNRELIABLE_CHANNEL_INDEX)
                    .try_send(message.to_packet(), host)
                    .is_err()
                {
                    error!("failed to send unreliable packet to {host}: {message:?}");
                }
            }
            if !queue.unreliable_to_host.is_empty() {
                trace!(
                    "sent {} [U] {} packets",
                    queue.unreliable_to_host.len(),
                    M::reflect_name()
                );
            }
        }
        queue.flush();
    }
}
