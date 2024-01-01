use bevy::prelude::Event;
use silk_common::AuthenticationRequest;

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
