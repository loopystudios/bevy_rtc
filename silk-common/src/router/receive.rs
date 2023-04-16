use crate::events::SocketRecvEvent;
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use silk_net::Message;

#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Message> {
    pub messages: Vec<(PeerId, M)>,
}

impl<M: Message> IncomingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general, this should be
    /// called once per frame/update.
    pub fn update(&mut self) {
        self.messages.clear();
    }

    /// A system that calls [`Events::update`] once per frame.
    pub fn update_system(mut incoming: ResMut<Self>) {
        incoming.update();
    }

    pub fn read_system(
        mut incoming: ResMut<Self>,
        mut events: EventReader<SocketRecvEvent>,
    ) {
        for SocketRecvEvent((peer_id, packet)) in events.iter() {
            if let Some(message) = M::from_packet(packet) {
                incoming.messages.push((*peer_id, message));
            }
        }
    }
}
