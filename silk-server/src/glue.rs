use axum::{extract::ws::Message, Error};
use serde::Deserialize;

use serde::Serialize;

pub type PeerId = String;

/// Requests go from peer to signalling server
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum PeerRequest<S> {
    Uuid(PeerId),
    Signal { receiver: PeerId, data: S },
    KeepAlive,
}

/// Events go from signalling server to peer
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum PeerEvent<S> {
    NewPeer(PeerId),
    PeerLeft(PeerId),
    Signal { sender: PeerId, data: S },
}

#[derive(Debug, Clone)]
pub(crate) struct Peer {
    pub uuid: PeerId,
    pub sender:
        tokio::sync::mpsc::UnboundedSender<std::result::Result<Message, Error>>,
}
