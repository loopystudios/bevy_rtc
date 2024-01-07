mod events;
mod router;
mod state;
mod system_params;
mod systems;

use crate::{
    events::SocketRecvEvent,
    packets::auth::{SilkLoginRequestPayload, SilkLoginResponsePayload},
    schedule::{SilkSchedule, SilkSet},
    socket::{common_socket_reader, SilkSocket},
};
use bevy::prelude::*;
use std::net::Ipv4Addr;

pub use events::SilkServerEvent;
pub use router::AddNetworkMessageExt;
pub use state::SilkState;
pub use system_params::{NetworkReader, NetworkWriter};

/// The socket server abstraction
pub struct SilkServerPlugin {
    /// Which port to serve the signaling server on
    pub port: u16,
    /// Hertz for [`SilkSchedule`](crate::schedule::SilkSchedule) tickrate,
    /// e.g. 30.0 = 30 times per second
    pub tick_rate: f64,
}

impl Plugin for SilkServerPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the schedule for silk
        app.init_schedule(SilkSchedule)
            .edit_schedule(SilkSchedule, |schedule| {
                schedule.configure_sets(SilkSet::sets());
            })
            .add_event::<SocketRecvEvent>()
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / self.tick_rate))
            .add_systems(FixedUpdate, |world: &mut World| {
                world.run_schedule(SilkSchedule);
            })
            .add_network_message::<SilkLoginRequestPayload>()
            .add_network_message::<SilkLoginResponsePayload>()
            .add_event::<SilkServerEvent>()
            .insert_resource(SilkState {
                addr: (Ipv4Addr::UNSPECIFIED, self.port).into(),
                id: None,
            })
            .add_systems(
                Startup,
                // We start a signaling server on localhost and the first peer
                // becomes host
                (systems::init_signaling_server, systems::init_server_socket)
                    .chain(),
            )
            .add_systems(
                SilkSchedule,
                systems::on_login.in_set(SilkSet::NetworkRead),
            )
            .add_systems(
                SilkSchedule,
                common_socket_reader
                    .run_if(resource_exists::<SilkSocket>())
                    .before(SilkSet::NetworkRead),
            )
            .add_systems(
                Update,
                systems::server_event_writer.after(systems::on_login),
            );
    }
}
