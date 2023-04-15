use bevy::prelude::*;

use super::message::Message;
#[derive(Default, Debug, Resource)]
pub struct NetworkQuery<M: Message> {
    messages: Vec<M>,
}

impl<M: Message> NetworkQuery<M> {
    /// Swaps the event buffers and clears the oldest event buffer. In general, this should be
    /// called once per frame/update.
    pub fn update(&mut self) {
        self.messages.clear();
    }

    /// A system that calls [`Events::update`] once per frame.
    pub fn update_system(mut query: ResMut<Self>) {
        query.update();
    }
}
