use silk_common::PlayerAuthentication;
use std::net::IpAddr;

pub enum ConnectionRequest {
    /// A request to connect to the server through the signaling server; the
    /// ip and port are the signaling server
    Connect {
        ip: IpAddr,
        port: u16,
        auth: PlayerAuthentication,
    },
    /// A request to disconnect from the signaling server; this will also
    /// disconnect from the server
    Disconnect { reason: Option<String> },
}
