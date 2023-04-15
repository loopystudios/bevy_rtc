use silk_common::{
    bevy_matchbox::matchbox_socket::{Packet, PeerId},
    packets::SilkPayload,
};

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone)]
pub enum SilkSocketEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// The socket has successfully connected to a host
    ConnectedToHost { host: PeerId, username: String },
    /// The socket disconnected from the host
    DisconnectedFromHost { reason: Option<String> },
    /// A message was received from the host
    Message((PeerId, Packet)),
}

/// Request events for client to send messages to server
#[derive(Debug)]
pub enum SilkSendEvent {
    /// Send an unreliable packet (any order, no retransmit) to the server
    UnreliableSend(SilkPayload),
    /// Send a reliable packet to the server
    ReliableSend(SilkPayload),
}
