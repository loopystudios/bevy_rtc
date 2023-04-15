use bevy::prelude::*;

mod message;
mod query;

pub use message::Message;
pub use query::NetworkQuery;

pub trait AddNetworkQuery {
    fn add_network_query<T: Message>(&mut self) -> &mut Self;
}

impl AddNetworkQuery for App {
    fn add_network_query<T>(&mut self) -> &mut Self
    where
        T: Message,
    {
        if !self.world.contains_resource::<NetworkQuery<T>>() {
            self.init_resource::<NetworkQuery<T>>()
                .add_system(
                    NetworkQuery::<T>::update_system
                        .in_base_set(CoreSet::First),
                )
                .add_system(
                    NetworkQuery::<T>::receive_system
                        .in_base_set(CoreSet::PreUpdate),
                );
        }
        self
    }
}
