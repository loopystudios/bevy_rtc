use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_matchbox::prelude::*;
use silk_client::{
    events::{SilkSendEvent, SilkSocketEvent},
    ConnectionRequest, SilkClientPlugin,
};
use std::{
    net::{IpAddr, Ipv4Addr},
    ops::DerefMut,
};

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
                    "error,chat_client=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
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
    .insert_resource(MessageScrollBox::default())
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
    mut messages: ResMut<MessageScrollBox>,
) {
    for event in events.iter() {
        match event {
            SilkSocketEvent::IdAssigned(id) => {
                info!("Got ID from signaling server: {id:?}");
                world_state.id.replace(*id);
            }
            SilkSocketEvent::ConnectedToHost(id) => {
                // Connected to host
                info!("Connected to host: {id:?}");
                _ = app_state.overwrite_set(ConnectionState::Connected);
            }
            SilkSocketEvent::DisconnectedFromHost => {
                // Disconnected from host
                error!("Disconnected from host");
                _ = app_state.overwrite_set(ConnectionState::Disconnected);
                *world_state = WorldState::default();
            }
            SilkSocketEvent::Message((peer, data)) => {
                let peer = *peer;
                let message = String::from_utf8_lossy(data).to_string();
                info!("{peer:?}: {message}");
                messages.messages.push((peer, message));
            }
        }
    }
}

fn ui_example_system(
    mut egui_context: ResMut<EguiContext>,
    mut event_wtr: EventWriter<ConnectionRequest>,
    world_state: Res<WorldState>,
    mut silk_event_wtr: EventWriter<SilkSendEvent>,
    mut messages: ResMut<MessageScrollBox>,
    mut text: Local<String>,
) {
    egui::Window::new("Login").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("{:?}", world_state.id));
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
    egui::Window::new("Chat").show(egui_context.ctx_mut(), |ui| {
        ui.label("Send Message");
        ui.horizontal_wrapped(|ui| {
            ui.text_edit_singleline(text.deref_mut());
            if ui.button("Send").clicked() {
                // TODO: Send chat line
                let data = text.as_bytes().to_vec().into_boxed_slice();
                silk_event_wtr.send(SilkSendEvent::ReliableSend(data));
            };
        });
        ui.label("Messages");
        messages.ui(ui);
    });
}

#[derive(Resource, Default, PartialEq)]
struct MessageScrollBox {
    messages: Vec<(PeerId, String)>,
}

use egui::*;
impl MessageScrollBox {
    fn ui(&mut self, ui: &mut Ui) {
        let text_style = egui::TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        ScrollArea::vertical().stick_to_bottom(true).show_rows(
            ui,
            row_height,
            self.messages.len(),
            |ui, items| {
                for i in items {
                    let (from, message) = &self.messages[i];
                    let text = format!("<-- {from:?}: {message}");
                    ui.label(text);
                }
            },
        );

        ui.ctx().request_repaint();
    }
}
