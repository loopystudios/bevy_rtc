use bevy::prelude::Resource;
use bevy_matchbox::prelude::PeerId;
use std::net::SocketAddr;

#[derive(Resource)]
pub struct SilkState {
    /// The socket address bound
    pub addr: SocketAddr,

    /// The ID the host (server)
    pub id: Option<PeerId>,
}
