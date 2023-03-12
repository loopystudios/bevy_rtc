use bevy::prelude::*;
use matchbox_socket::PeerId;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct ServerState {
    pub clients: HashSet<PeerId>,
}
