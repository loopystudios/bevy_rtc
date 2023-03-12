use matchbox_socket::PeerId;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct ServerState {
    pub clients: HashSet<PeerId>,
}
