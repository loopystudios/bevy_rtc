use matchbox_socket::{Packet, PeerId};

pub enum SilkServerEvent {
    PeerJoined(PeerId),
    PeerLeft(PeerId),
    MessageReceived(Packet),
}

pub enum SilkBroadcastEvent {
    UnreliableSendAll(Packet),
    UnreliableSend((PeerId, Packet)),
    ReliableSendAll(Packet),
    ReliableSend((PeerId, Packet)),
}
