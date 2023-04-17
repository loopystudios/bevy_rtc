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
        for SocketRecvEvent((_, packet)) in events.iter() {
            if let Some(message) = M::from_packet(packet) {
                incoming.messages.push(message);
            }
        }
    }
}
