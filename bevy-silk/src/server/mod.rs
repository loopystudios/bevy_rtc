mod events;
mod router;
mod signaler;
mod state;
mod system_params;
mod systems;

use crate::{
    common_plugin::SilkCommonPlugin,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::{SilkSchedule, SilkSet},
};
use bevy::prelude::*;
use signaler::SilkSignalerPlugin;
use std::net::Ipv4Addr;

pub use events::SilkServerEvent;
pub use router::AddNetworkMessageExt;
pub use state::SilkState;
pub use system_params::{NetworkReader, NetworkWriter};

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Which port to serve the signaling server on
    pub port: u16,
    /// Hertz for server tickrate, e.g. 30.0 = 30 times per second
    pub tick_rate: f64,
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SilkSignalerPlugin { port: self.port });

        app.add_plugins(SilkCommonPlugin)
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .add_event::<SilkServerEvent>()
            .insert_resource(SilkState {
                addr: format!("ws://{}:{}", Ipv4Addr::LOCALHOST, self.port),
                id: None,
            })
            .add_systems(Startup, systems::init_socket)
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / self.tick_rate));

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
