use matchbox_socket::{Packet, PeerId};

/// Socket events that are possible to subscribe to in Bevy
pub enum SilkServerEvent {
    /// The signalling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// A peer has connected to this server
    PeerJoined(PeerId),
    /// A peer has left this server
    PeerLeft(PeerId),
    /// A peer sent a message to this server
    MessageReceived((PeerId, Packet)),
}

/// Request events for the server to broadcast a message
pub enum SilkBroadcastEvent {
    /// Send an unreliable packet (any order, no retransmit) to all peers
    UnreliableSendAll(Packet),
    /// Send an unreliable packet (any order, no retransmit) to a peer
    UnreliableSend((PeerId, Packet)),
    /// Send a reliable packet to all peers
    ReliableSendAll(Packet),
    /// Send a reliable packet to a peer
    ReliableSend((PeerId, Packet)),
}
