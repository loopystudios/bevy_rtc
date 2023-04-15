use bevy_matchbox::matchbox_socket::Packet;
use bincode::ErrorKind;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SilkPayload {
    AuthenticateUser {
        username: String,
        password: String,
        mfa: Option<String>,
    },
    AuthenticateGuest {
        username: String,
    },
    LoginAccepted {
        username: String,
    },
    LoginDenied {
        reason: String,
    },
    Message(Packet),
}

impl SilkPayload {
    /// Convert this game payload into a packet.
    pub fn into_packet(&self) -> Packet {
        Packet::try_from(self).unwrap()
    }
}

impl From<&SilkPayload> for Packet {
    fn from(value: &SilkPayload) -> Self {
        bincode::serialize(value).unwrap().into_boxed_slice()
    }
}

impl TryFrom<&Packet> for SilkPayload {
    type Error = Box<ErrorKind>;

    fn try_from(value: &Packet) -> Result<Self, Self::Error> {
        bincode::deserialize(value)
    }
}
