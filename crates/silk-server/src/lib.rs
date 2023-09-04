use bevy::{prelude::*, time::fixed_timestep::FixedTime};
pub use router::{AddNetworkMessageExt, IncomingMessages, OutgoingMessages};
use signaler::SilkSignalerPlugin;
use silk_common::{
    events::SilkServerEvent,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::*,
    sets::SilkSet,
    SilkCommonPlugin,
};
use state::ServerState;
use std::net::Ipv4Addr;
pub use system_params::{NetworkReader, NetworkWriter};

mod router;
pub mod signaler;
pub(crate) mod state;
mod system_params;
pub(crate) mod systems;

/// Configuration used for server signaling
pub enum SignalingConfig {
    /// I am hosting my own signaler
    Local { port: u16 },
    /// I am using a remote signaler
    Remote { addr: String },
}

impl SignalingConfig {
    pub fn to_url(&self) -> String {
        match self {
            SignalingConfig::Local { port } => {
                format!("ws://{}:{}", Ipv4Addr::LOCALHOST, port)
            }
            SignalingConfig::Remote { addr } => addr.to_owned(),
        }
    }
}

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Whether the signaling server is local or remote
    pub signaling: SignalingConfig,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f32,
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        if let SignalingConfig::Local { port } = self.signaling {
            app.add_plugins(SilkSignalerPlugin { port });
        }

        app.add_plugins(SilkCommonPlugin)
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .add_event::<SilkServerEvent>()
            .insert_resource(ServerState {
                addr: self.signaling.to_url(),
                id: None,
            })
            .add_systems(Startup, systems::init_socket)
            .insert_resource(FixedTime::new_from_secs(1.0 / self.tick_rate));

        app.add_systems(
            SilkSchedule,
            systems::on_login.in_set(SilkSet::NetworkRead),
        )
        .add_systems(
            SilkSchedule,
            // Read silk events always before servers, who hook into this stage
            systems::server_socket_reader
                .before(SilkSet::SilkEvents)
                .after(systems::on_login),
        );
    }
}
