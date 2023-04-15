use bevy::prelude::*;

use crate::events::RecvMessageEvent;

use super::message::Message;
#[derive(Default, Debug, Resource)]
pub struct IncomingMessages<M: Message> {
    pub messages: Vec<M>,
}

impl<M: Message> IncomingMessages<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general, this should be
    /// called once per frame/update.
    pub fn update(&mut self) {
        self.messages.clear();
    }

    /// A system that calls [`Events::update`] once per frame.
    pub fn update_system(mut query: ResMut<Self>) {
        query.update();
    }

    pub fn read_system(
        mut query: ResMut<Self>,
        mut recv: EventReader<RecvMessageEvent>,
    ) {
        for RecvMessageEvent(_peer_id, packet) in recv.iter() {
            if let Some(message) = M::from_packet(packet) {
                error!("router received msg id {}", M::id());

                query.messages.push(message);
            }
        }
    }
}
