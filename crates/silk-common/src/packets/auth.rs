use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// Name hacking to get the import correct for silk_net::Payload.
mod bevy_silk {
    pub use silk_net as net;
}

#[derive(silk_net::Payload, Serialize, Deserialize, Debug, Clone)]
pub enum SilkLoginRequestPayload {
    RegisteredUser {
        access_token: String,
        character: String,
    },
    Guest {
        username: Option<String>,
    },
}

#[derive(silk_net::Payload, Serialize, Deserialize, Debug, Clone)]
pub enum SilkLoginResponsePayload {
    Accepted { username: String },
    Denied { reason: Option<String> },
}
