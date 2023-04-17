use bevy::prelude::*;
use silk_common::{bevy_matchbox::prelude::PeerId, events::SocketRecvEvent};
use silk_net::Message;

#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Message> {
    pub messages: Vec<(PeerId, M)>,
}

impl<M: Message> IncomingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general,
    /// this should be called once per frame/update.
    pub fn flush(mut incoming: ResMut<Self>) {
        incoming.messages.clear();
    }

    pub fn read_system(
        mut incoming: ResMut<Self>,
        mut events: EventReader<SocketRecvEvent>,
    ) {
        if !events.is_empty() {
            trace!("received {} {} packets", events.len(), M::reflect_name());
        }
        for SocketRecvEvent((peer_id, packet)) in events.iter() {
            if let Some(message) = M::from_packet(packet) {
                incoming.messages.push((*peer_id, message));
            }
        }
    }
}
