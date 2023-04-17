use bevy::prelude::*;
use silk_common::events::SocketRecvEvent;
use silk_net::Message;

#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Message> {
    pub messages: Vec<M>,
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
        for SocketRecvEvent((_, packet)) in events.iter() {
            if let Some(message) = M::from_packet(packet) {
                incoming.messages.push(message);
            }
        }
    }
}
