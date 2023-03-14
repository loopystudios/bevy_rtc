use crate::{error::ServerError, glue::Peer};
use axum::extract::ws::Message;
use matchbox_socket::PeerId;
use std::collections::HashMap;
use tracing::info;

#[derive(Default, Debug, Clone)]
pub(crate) struct ServerState {
    pub host: Option<Peer>,
    pub clients: HashMap<PeerId, Peer>,
}

impl ServerState {
    /// Reset the server state
    pub fn reset(&mut self) {
        self.host.take();
        self.clients.clear();
        info!("Server reset");
    }

    /// Add a clients, returning the peers already in room
    pub fn add_client(&mut self, peer: Peer) -> Vec<PeerId> {
        let existing_clients = self.clients.keys().cloned().collect();
        self.clients.insert(peer.uuid.clone(), peer);
        existing_clients
    }

    /// Remove a client from the state if it existed, returning the client
    /// removed.
    #[must_use]
    pub fn remove_client(&mut self, peer_id: &PeerId) -> Option<Peer> {
        self.clients.remove(peer_id)
    }

    /// Send a message to a client without blocking.
    pub fn try_send(
        &self,
        id: &PeerId,
        message: Message,
    ) -> Result<(), ServerError> {
        let peer = self.clients.get(id);
        let peer = match peer {
            Some(peer) => peer,
            None => {
                return Err(ServerError::UnknownPeer);
            }
        };

        peer.sender.send(Ok(message)).map_err(ServerError::from)
    }

    /// Send a message to a client without blocking.
    pub fn try_send_to_host(
        &self,
        message: Message,
    ) -> Result<(), ServerError> {
        let peer = self.host.as_ref();
        let peer = match peer {
            Some(peer) => peer,
            None => {
                return Err(ServerError::UnknownPeer);
            }
        };

        peer.sender.send(Ok(message)).map_err(ServerError::from)
    }
}
