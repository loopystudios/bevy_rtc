use matchbox_socket::PeerState;

pub enum SilkSocketEvent {
    IdAssigned(String),
    IdRemoved,
    PeerStateChange((String, PeerState)),
    Message((String, Box<[u8]>)),
}
