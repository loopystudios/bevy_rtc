use axum::{extract::ws::Message, Error};
use matchbox_socket::PeerId;

#[derive(Debug, Clone)]
pub(crate) struct Peer {
    pub uuid: PeerId,
    pub sender:
        tokio::sync::mpsc::UnboundedSender<std::result::Result<Message, Error>>,
}
pub type PeerRequest = matchbox::PeerRequest<serde_json::Value>;
pub type PeerEvent = matchbox::PeerEvent<serde_json::Value>;

mod matchbox {
    use matchbox_socket::PeerId;
    use serde::{Deserialize, Serialize};

    /// Requests go from peer to signalling server
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum PeerRequest<S> {
        Signal { receiver: PeerId, data: S },
        KeepAlive,
    }

    /// Events go from signalling server to peer
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum PeerEvent<S> {
        IdAssigned(PeerId),
        NewPeer(PeerId),
        PeerLeft(PeerId),
        Signal { sender: PeerId, data: S },
    }
}
