pub enum SilkSocketEvent {
    IdAssigned(String),
    ConnectedToHost(String),
    DisconnectedFromHost,
    Message((String, Box<[u8]>)),
}
