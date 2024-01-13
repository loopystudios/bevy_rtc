use bevy::prelude::Event;
use bevy_matchbox::matchbox_socket::PeerId;

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone, Event)]
pub enum RtcClientEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// The socket has successfully connected to a host
    ConnectedToHost(PeerId),
    /// The socket disconnected from the host
    DisconnectedFromHost { reason: Option<String> },
}

#[derive(Debug, Clone, Event)]
pub enum ConnectionRequest {
    /// A request to connect to the server through the signaling server.
    /// The format of the addr should be ws://host:port or wss://host:port
    Connect { addr: String },
    /// A request to fully disconnect
    Disconnect,
}
