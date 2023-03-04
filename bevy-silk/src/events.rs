pub enum SilkSocketEvent {
    IdAssigned(String),
    IdRemoved,
    ConnectedToHost(String),
    DisconnectedFromHost(String),
    Message((String, Box<[u8]>)),
}
