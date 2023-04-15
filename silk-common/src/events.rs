use bevy_matchbox::matchbox_socket::{Packet, PeerId};

#[derive(Debug, Clone)]
pub struct SocketRecvEvent(pub (PeerId, Packet));

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone)]
pub enum SilkSocketEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// The socket has successfully connected to a host
    ConnectedToHost(PeerId),
    /// The socket disconnected from the host
    DisconnectedFromHost,
    /// A peer has connected to this server
    ClientJoined(PeerId),
    /// A peer has left this server
    ClientLeft(PeerId),
}
