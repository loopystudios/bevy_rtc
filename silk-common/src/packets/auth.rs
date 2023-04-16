use proc_macro_payload::Payload;
use serde::{Deserialize, Serialize};

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct SilkAuthUserPayload {
    pub username: String,
    pub password: String,
    pub mfa: Option<String>,
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct SilkAuthGuestPayload {
    pub username: Option<String>,
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct SilkLoginAcceptedPayload {
    pub username: String,
}

#[derive(Payload, Serialize, Deserialize, Debug, Clone, Default)]
pub struct SilkLoginDeniedPayload {
    pub reason: Option<String>,
}
