use bevy::{ecs::system::SystemParam, prelude::*};

use crate::events::RecvMessageEvent;

#[derive(SystemParam, Debug)]
pub struct NetworkQuery<'w, M: Message> {
    received: Res<'w, RecvMessages<M>>,
}

impl<'w, M: Message> NetworkQuery<'w, M> {
    pub fn iter(&mut self) -> std::slice::Iter<'_, M> {
        self.received.messages.iter()
    }
}

use super::message::Message;
#[derive(Default, Debug, Resource)]
pub struct RecvMessages<M: Message> {
    pub messages: Vec<M>,
}

impl<M: Message> RecvMessages<M> {
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
                error!("NetworkQuery received!");

                query.messages.push(message);
            }
        }
    }
}
