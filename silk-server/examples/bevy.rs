use bevy::{log::LogPlugin, prelude::*};
use silk_server::{
    events::{SilkBroadcastEvent, SilkServerEvent},
    SilkServerPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=debug"
                .into(),
            level: bevy::log::Level::DEBUG,
        })
        .add_plugin(SilkServerPlugin {
            port: 3536,
            tick_rate: 5.0,
            signalling_server: None,
        })
        .add_system_to_stage(
            silk_server::stages::PROCESS_INCOMING_EVENTS,
            handle_events,
        )
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
