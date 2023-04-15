use silk_common::{
    bevy_matchbox::matchbox_socket::{Packet, PeerId},
    packets::SilkPayload,
};

/// Socket events that are possible to subscribe to in Bevy
#[derive(Debug, Clone)]
pub enum SilkServerEvent {
    /// The signaling server assigned the socket a unique ID
    IdAssigned(PeerId),
    GuestLoginRequest {
        peer_id: PeerId,
        username: String,
    },
    LoginRequest {
        peer_id: PeerId,
        username: String,
        password: String,
        mfa: Option<String>,
    },
    /// A peer has left this server
    ClientLeft(PeerId),
    /// A peer sent a message to this server
    Message((PeerId, Packet)),
}

/// Request events for the server to broadcast a message
#[derive(Debug)]
pub(crate) enum SilkRawBroadcastEvent {
    /// Send a raw packet to a peer
    UnreliableSend((PeerId, Packet)),
    /// Send a raw packet to a peer
    ReliableSend((PeerId, Packet)),
}

/// Request events for the server to broadcast a message
#[derive(Debug)]
pub enum SilkBroadcastEvent {
    /// Send an unreliable packet (any order, no retransmit) to all peers except one
    UnreliableSendAllExcept((PeerId, SilkPayload)),
    /// Send an unreliable packet (any order, no retransmit) to all peers
    UnreliableSendAll(SilkPayload),
    /// Send an unreliable packet (any order, no retransmit) to a peer
    UnreliableSend((PeerId, SilkPayload)),

    /// Send a reliable packet to all peers except one
    ReliableSendAllExcept((PeerId, SilkPayload)),
    /// Send a reliable packet to all peers
    ReliableSendAll(SilkPayload),
    /// Send a reliable packet to a peer
    ReliableSend((PeerId, SilkPayload)),
}
