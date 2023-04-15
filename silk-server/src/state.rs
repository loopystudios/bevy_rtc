use bevy::prelude::*;
use silk_common::{bevy_matchbox::prelude::PeerId, ConnectionAddr};

#[derive(Resource)]
pub struct SocketState {
    /// The socket address, used for connecting/reconnecting
    pub addr: ConnectionAddr,

    /// The ID the signaling server sees us as
    pub id: Option<PeerId>,
}
