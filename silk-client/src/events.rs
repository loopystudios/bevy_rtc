use matchbox_socket::{Packet, PeerId};

/// Socket events that are possible to subscribe to in Bevy
pub enum SilkSocketEvent {
    /// The signalling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// The socket has successfully connected to a host
    ConnectedToHost(PeerId),
    /// The socket disconnected from the host
    DisconnectedFromHost,
    /// A message was received from the host
    Message((PeerId, Packet)),
}

/// Request events for client to send messages to server
pub enum SilkSendEvent {
    /// Send an unreliable packet (any order, no retransmit) to the server
    UnreliableSend(Packet),
    /// Send a reliable packet to the server
    ReliableSend(Packet),
}
