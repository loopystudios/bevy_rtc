use crate::router::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkAuthUserPayload {
    pub username: String,
    pub password: String,
    pub mfa: Option<String>,
}

impl Message for SilkAuthUserPayload {
    fn id() -> u16 {
        100
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkAuthGuestPayload {
    pub username: Option<String>,
}

impl Message for SilkAuthGuestPayload {
    fn id() -> u16 {
        101
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkLoginAcceptedPayload {
    pub username: String,
}

impl Message for SilkLoginAcceptedPayload {
    fn id() -> u16 {
        102
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SilkLoginDeniedPayload {
    pub reason: Option<String>,
}

impl Message for SilkLoginDeniedPayload {
    fn id() -> u16 {
        103
    }
}
