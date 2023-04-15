use silk_common::bevy_matchbox::matchbox_socket::{Packet, PeerId};

/// Request events for the server to broadcast a message
#[derive(Debug)]
pub enum SilkBroadcastEvent {
    /// Send an unreliable packet (any order, no retransmit) to all peers except one
    UnreliableSendAllExcept((PeerId, Packet)),
    /// Send an unreliable packet (any order, no retransmit) to all peers
    UnreliableSendAll(Packet),
    /// Send an unreliable packet (any order, no retransmit) to a peer
    UnreliableSend((PeerId, Packet)),

    /// Send a reliable packet to all peers except one
    ReliableSendAllExcept((PeerId, Packet)),
    /// Send a reliable packet to all peers
    ReliableSendAll(Packet),
    /// Send a reliable packet to a peer
    ReliableSend((PeerId, Packet)),
}
