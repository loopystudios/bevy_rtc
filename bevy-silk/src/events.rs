pub enum SilkSocketEvent {
    IdAssigned(String),
    ConnectedToHost(String),
    DisconnectedFromHost(String),
    Message((String, Box<[u8]>)),
}
