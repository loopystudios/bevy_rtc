use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use matchbox_socket::PeerId;
use silk_client::{
    events::SilkSocketEvent, ConnectionRequest, SilkClientPlugin,
};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ConnectionState {
    Disconnected,
    Connected,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldState {
    id: Option<PeerId>,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter:
                    "error,client=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
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
    .add_state(ConnectionState::Disconnected)
    .insert_resource(WorldState::default())
    .add_system(handle_events)
    .add_system_set(
        SystemSet::on_enter(ConnectionState::Disconnected)
            .with_system(on_disconnected),
    )
    .add_system_set(
        SystemSet::on_enter(ConnectionState::Connected)
            .with_system(on_connected),
    )
    .add_startup_system(setup_cam)
    .add_plugin(EguiPlugin)
    .add_system(ui_example_system)
    .run();
}

fn setup_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn on_disconnected(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::RED));
}

fn on_connected(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::GREEN));
}

fn handle_events(
    mut app_state: ResMut<State<ConnectionState>>,
    mut events: EventReader<SilkSocketEvent>,
    mut world_state: ResMut<WorldState>,
) {
    for event in events.iter() {
        match event {
            SilkSocketEvent::IdAssigned(id) => {
                info!("Got ID from signalling server: {id}");
                world_state.id.replace(id.clone());
            }
            SilkSocketEvent::ConnectedToHost(id) => {
                // Connected to host
                info!("Connected to host: {id}");
                app_state.set(ConnectionState::Connected).unwrap();
            }
            SilkSocketEvent::DisconnectedFromHost => {
                // Disconnected from host
                error!("Disconnected from host");
                app_state.set(ConnectionState::Disconnected).unwrap();
                *world_state = WorldState::default();
            }
            SilkSocketEvent::Message((peer, data)) => {
                info!("Message from {peer}: {}", String::from_utf8_lossy(data));
            }
        }
    }
}

fn ui_example_system(
    mut egui_context: ResMut<EguiContext>,
    mut event_wtr: EventWriter<ConnectionRequest>,
    world_state: Res<WorldState>,
) {
    egui::Window::new("Debug").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("I am {:?}", world_state.id));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Connect").clicked() {
                event_wtr.send(ConnectionRequest::Connect {
                    ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    port: 3536,
                });
            }
            if ui.button("Disconnect").clicked() {
                event_wtr.send(ConnectionRequest::Disconnect);
            }
        });
    });
}
