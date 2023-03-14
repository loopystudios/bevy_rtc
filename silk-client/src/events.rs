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
