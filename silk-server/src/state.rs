use bevy::prelude::Resource;
use silk_common::{bevy_matchbox::prelude::PeerId, ConnectionAddr};

#[derive(Resource)]
pub struct ServerState {
    /// The socket address, used for connecting/reconnecting
    pub addr: ConnectionAddr,

    /// The ID the signaling server sees us as
    pub id: Option<PeerId>,
}
