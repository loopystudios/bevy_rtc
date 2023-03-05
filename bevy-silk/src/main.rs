use std::net::{IpAddr, Ipv4Addr};

use bevy::{log::LogPlugin, prelude::*};
use bevy_silk::{
    events::SilkSocketEvent, ConnectToRemoteHostEvent, SilkClientPlugin,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Connecting,
    InGame,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter:
                    "info,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=debug"
                        .into(),
                level: bevy::log::Level::DEBUG,
            })
            .set(WindowPlugin {
                window: WindowDescriptor {
                    fit_canvas_to_parent: true, // behave on wasm
                    ..default()
                },
                ..default()
            }),
    )
    .add_plugin(SilkClientPlugin)
    .add_state(AppState::Connecting)
    .add_system(handle_events)
    .add_system_set(
        SystemSet::on_enter(AppState::Connecting).with_system(on_connecting),
    )
    .add_system_set(
        SystemSet::on_enter(AppState::InGame).with_system(on_connected),
    )
    .add_startup_system(setup_cam)
    .add_startup_system(setup_networking)
    .run();
}

fn setup_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_networking(mut event_wtr: EventWriter<ConnectToRemoteHostEvent>) {
    // Send one connect-to-host "request" (bevy event) on startup to the Silk
    // Client plugin with the desired host description
    event_wtr.send(ConnectToRemoteHostEvent {
        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 3536,
    });
}

fn on_connecting(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::RED));
}

fn on_connected(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::GREEN));
}

fn handle_events(
    mut app_state: ResMut<State<AppState>>,
    mut events: EventReader<SilkSocketEvent>,
) {
    for event in events.iter() {
        match event {
            SilkSocketEvent::IdAssigned(id) => {
                info!("Got ID from signalling server: {id}")
            }
            SilkSocketEvent::IdRemoved => error!("Lost ID"),
            SilkSocketEvent::ConnectedToHost(id) => {
                // Connected to host
                info!("Connected to host: {id}");
                app_state.set(AppState::InGame).unwrap();
            }
            SilkSocketEvent::DisconnectedFromHost(id) => {
                // Disconnected from host
                error!("Disconnected from host: {id}");
                app_state.set(AppState::Connecting).unwrap();
            }
            SilkSocketEvent::Message((peer, data)) => {
                info!("message from {peer}: {}", String::from_utf8_lossy(data));
            }
        }
    }
}
