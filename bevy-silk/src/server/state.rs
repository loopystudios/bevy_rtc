use bevy::prelude::Resource;
use bevy_matchbox::prelude::PeerId;

#[derive(Resource)]
pub struct SilkState {
    /// The socket address bound
    pub addr: String,

    /// The ID the host (server)
    pub id: Option<PeerId>,
}
