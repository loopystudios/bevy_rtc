use matchbox_socket::PeerState;

pub enum SocketEvent {
    IdAssigned(String),
    IdRemoved,
    PeerStateChange((String, PeerState)),
    Message((String, Box<[u8]>)),
}
