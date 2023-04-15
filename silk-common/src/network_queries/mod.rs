use bevy::prelude::*;
use serde::Deserialize;

pub trait Message:
    for<'a> Deserialize<'a> + std::default::Default + Send + Sync + 'static
{
}

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

pub trait AppAddNetworkQuery {
    fn add_network_query<T: Message>(&mut self) -> &mut Self;
}

impl AppAddNetworkQuery for App {
    fn add_network_query<T>(&mut self) -> &mut Self
    where
        T: Message,
    {
        if !self.world.contains_resource::<NetworkQuery<T>>() {
            self.init_resource::<NetworkQuery<T>>().add_system(
                NetworkQuery::<T>::update_system.in_base_set(CoreSet::First),
            );
        }
        self
    }
}
