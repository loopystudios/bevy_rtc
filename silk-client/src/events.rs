use silk_common::bevy_matchbox::matchbox_socket::{Packet, PeerId};

/// Request events for client to send messages to server
#[derive(Debug)]
pub enum SilkSendEvent {
    /// Send an unreliable packet (any order, no retransmit) to the server
    UnreliableSend(Packet),
    /// Send a reliable packet to the server
    ReliableSend(Packet),
}
