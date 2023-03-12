use bevy::{log::LogPlugin, prelude::*};

use silk_common::SilkSocketConfig;
use silk_server::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    SilkServerPlugin, SocketResource,
};
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter:
            "info,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=debug".into(),
        level: bevy::log::Level::DEBUG,
    }))
    .add_plugin(SilkServerPlugin {
        port: 3536,
        tick_rate: 5.0,
    })
    .add_system(handle_events)
    .run();
}

fn handle_events(
    mut event_rdr: EventReader<SilkServerEvent>,
    mut event_wtr: EventWriter<SilkBroadcastEvent>,
) {
    while let Some(ev) = event_rdr.iter().next() {
        match ev {
            SilkServerEvent::PeerJoined(_) => {
                info!("someone joined");
                let packet =
                    "someone joined!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
            SilkServerEvent::PeerLeft(_) => {
                info!("someone left");
                let packet =
                    "someone left!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
            SilkServerEvent::MessageReceived(_) => {
                info!("you got mail");
                let packet =
                    "message received!".as_bytes().to_vec().into_boxed_slice();
                event_wtr.send(SilkBroadcastEvent::ReliableSendAll(packet));
            }
        }
    }
    event_rdr.clear();
}
