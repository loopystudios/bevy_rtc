mod chat;
mod painting;

use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_egui::{
    egui::{self, Pos2},
    EguiContexts, EguiPlugin,
};
use bevy_silk::{
    client::{
        AddNetworkMessageExt, ConnectionRequest, NetworkReader, NetworkWriter,
        SilkClientEvent, SilkClientPlugin, SilkConnectionState, SilkState,
    },
    protocol::AuthenticationRequest,
    schedule::{SilkSchedule, SilkSet},
};
use chat::ChatState;
use painting::PaintingState;
use protocol::{ChatPayload, DrawLinePayload};
use std::ops::DerefMut;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()).set(
            WindowPlugin {
                primary_window: Some(bevy::window::Window {
                    present_mode: PresentMode::AutoVsync,
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    resolution: WindowResolution::new(400., 700.),
                    ..default()
                }),
                ..default()
            },
        ))
        .add_plugins(EguiPlugin)
        .add_plugins(SilkClientPlugin)
        .add_network_message::<ChatPayload>()
        .add_network_message::<DrawLinePayload>()
        .add_systems(Update, handle_events)
        .add_systems(SilkSchedule, read_chats.in_set(SilkSet::NetworkRead))
        .add_systems(SilkSchedule, read_lines.in_set(SilkSet::NetworkRead))
        .add_systems(SilkSchedule, send_chats.in_set(SilkSet::NetworkWrite))
        .add_systems(SilkSchedule, send_lines.in_set(SilkSet::NetworkWrite))
        .add_systems(Update, app_ui)
        .add_systems(
            OnEnter(SilkConnectionState::Disconnected),
            on_disconnected,
        )
        .add_systems(OnEnter(SilkConnectionState::Establishing), on_logging_in)
        .add_systems(OnEnter(SilkConnectionState::Connected), on_connected)
        .add_systems(Startup, setup_cam)
        .insert_resource(ChatState::default())
        .insert_resource(PaintingState::default())
        .run();
}

fn setup_cam(mut commands: Commands) {
    // Without a camera we get no clear color
    commands.spawn(Camera2dBundle::default());
}

fn on_disconnected(
    mut commands: Commands,
    mut chat_state: ResMut<ChatState>,
    mut painting_state: ResMut<PaintingState>,
) {
    commands.insert_resource(ClearColor(Color::RED));
    *chat_state = ChatState::default();
    *painting_state = PaintingState::default();
}

fn on_logging_in(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::ORANGE));
}

fn on_connected(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::GREEN));
}

fn handle_events(mut events: EventReader<SilkClientEvent>) {
    for ev in events.read() {
        debug!("event: {ev:?}");
        match ev {
            SilkClientEvent::IdAssigned(id) => {
                info!("ID assigned: {id:?}");
            }
            SilkClientEvent::ConnectedToHost { host, username } => {
                // Connected to host
                info!("Connected to host ({host}) as {username}");
            }
            SilkClientEvent::DisconnectedFromHost { reason } => {
                // Disconnected from host
                warn!("Disconnected from host, reason: {reason:?}");
            }
        }
    }
}

fn read_chats(
    mut chat_state: ResMut<ChatState>,
    mut chat_read: NetworkReader<ChatPayload>,
) {
    for chat in chat_read.iter() {
        chat_state.messages.insert(0, chat.to_owned());
    }
}

fn send_chats(
    mut chat_state: ResMut<ChatState>,
    mut chat_send: NetworkWriter<ChatPayload>,
    silk_state: Res<SilkState>,
) {
    if let Some(message) = chat_state.out.take() {
        let payload = ChatPayload {
            from: silk_state.id.unwrap().to_string(),
            message,
        };
        chat_send.reliable_to_host(payload);
    }
}

fn read_lines(
    mut painting_state: ResMut<PaintingState>,
    mut painting_read: NetworkReader<DrawLinePayload>,
) {
    for draw in painting_read.iter() {
        let DrawLinePayload { x1, y1, x2, y2 } = draw;
        painting_state
            .lines
            .push(vec![Pos2::new(*x1, *y1), Pos2::new(*x2, *y2)]);
    }
}

fn send_lines(
    mut painting_state: ResMut<PaintingState>,
    mut painting_send: NetworkWriter<DrawLinePayload>,
) {
    let draws = painting_state.out.drain(..);
    for (x1, y1, x2, y2) in draws {
        let draw = DrawLinePayload { x1, y1, x2, y2 };
        painting_send.unreliable_to_host(draw)
    }
}

#[allow(clippy::too_many_arguments)]
fn app_ui(
    state: Res<SilkState>,
    connection_status: Res<State<SilkConnectionState>>,
    mut contexts: EguiContexts,
    mut painting_state: ResMut<PaintingState>,
    mut connection_requests: EventWriter<ConnectionRequest>,
    mut chat_state: ResMut<ChatState>,
    mut room_url: Local<String>,
    mut chat_line: Local<String>,
) {
    let window = egui::Window::new("Painting Demo")
        .pivot(egui::Align2::CENTER_CENTER)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .resizable(false)
        .title_bar(true)
        .collapsible(false);
    window.show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| match connection_status.get() {
            SilkConnectionState::Establishing
            | SilkConnectionState::Authenticating
            | SilkConnectionState::Disconnected => {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Room URL:");
                    ui.add(
                        egui::TextEdit::singleline(&mut *room_url)
                            .hint_text("ws://127.0.0.1:3536 (default)")
                            .password(true)
                            .desired_width(300.0),
                    );
                });
                if ui.button("Connect").clicked() {
                    let auth = AuthenticationRequest::Guest { username: None };
                    connection_requests.send(ConnectionRequest::Connect {
                        addr: if room_url.is_empty() {
                            "ws://127.0.0.1:3536".to_string()
                        } else {
                            room_url.to_string()
                        },
                        auth,
                    });
                }
            }
            SilkConnectionState::Connected => {
                ui.horizontal(|ui| {
                    if ui.button("Disconnect").clicked() {
                        connection_requests.send(
                            ConnectionRequest::Disconnect {
                                reason: Some(
                                    "User clicked disconnect".to_string(),
                                ),
                            },
                        );
                    }
                    ui.label(format!("Connected as {}", state.id.unwrap()));
                });
                ui.separator();

                // Chat UI
                ui.label("Chat");
                ui.horizontal_wrapped(|ui| {
                    ui.text_edit_singleline(chat_line.deref_mut());
                    if ui.button("Send").clicked() {
                        chat_state.out.replace(chat_line.to_owned());
                    };
                });
                ui.label("Messages");
                chat_state.ui(ui);

                ui.separator();

                // Paint GUI
                let mut out: Option<(f32, f32, f32, f32)> = None;
                painting_state.ui(ui, &mut out);
                if let Some(draw) = out {
                    painting_state.out.push(draw);
                }
            }
        });
    });
}
