use bevy::prelude::Event;
use bevy_matchbox::matchbox_socket::PeerId;

use crate::protocol::AuthenticationRequest;

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone, Event)]
pub enum SilkClientEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    /// The socket has successfully connected to a host
    ConnectedToHost {
        /// The PeerId of the host that authenticated you
        host: PeerId,
        /// The username the host gives you
        username: String,
    },
    /// The socket disconnected from the host
    DisconnectedFromHost { reason: Option<String> },
}

#[derive(Debug, Clone, Event)]
pub enum ConnectionRequest {
    /// A request to connect to the server through the signaling server.
    /// The format of the addr should be ws://host:port or wss://host:port
    Connect {
        addr: String,
        auth: AuthenticationRequest,
    },
    /// A request to disconnect from the signaling server; this will also
    /// disconnect from the server
    Disconnect { reason: Option<String> },
}
