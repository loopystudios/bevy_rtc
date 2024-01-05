use bevy::prelude::Resource;
use bevy_matchbox::prelude::PeerId;

#[derive(Resource)]
pub struct ServerState {
    /// The socket address, used for connecting/reconnecting
    pub addr: String,

    /// The ID the signaling server sees us as
    pub id: Option<PeerId>,
}
