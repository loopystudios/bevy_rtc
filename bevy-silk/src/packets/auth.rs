use crate::protocol::Payload;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// Name hacking to get the import correct for the `Payload` macro.
mod bevy_silk {
    pub use crate::*;
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone)]
pub enum SilkLoginRequestPayload {
    RegisteredUser {
        access_token: String,
        character: String,
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
