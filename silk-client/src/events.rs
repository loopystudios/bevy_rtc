use silk_common::bevy_matchbox::matchbox_socket::{Packet, PeerId};

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone)]
pub enum SilkSocketEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// The socket has successfully connected to a host
    ConnectedToHost(PeerId),
    /// The socket disconnected from the host
    DisconnectedFromHost,
    /// A message was received from the host
    Message((PeerId, Packet)),
}

/// Request events for client to send messages to server
#[derive(Debug)]
pub enum SilkSendEvent {
    /// Send an unreliable packet (any order, no retransmit) to the server
    UnreliableSend(Packet),
    /// Send a reliable packet to the server
    ReliableSend(Packet),
}
