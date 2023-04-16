use crate::macros::Payload;
use serde::{Deserialize, Serialize};

#[derive(Payload, Serialize, Deserialize, Debug, Clone)]
pub enum SilkLoginRequestPayload {
    RegisteredUser {
        username: String,
        password: String,
        mfa: Option<String>,
    },
    Guest {
        username: Option<String>,
    },
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone)]
pub enum SilkLoginResponsePayload {
    Accepted { username: String },
    Denied { reason: Option<String> },
}
