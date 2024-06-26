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
use bevy_rtc::prelude::*;
use chat::ChatState;
use painting::PaintingState;
use protocol::{ChatPayload, DrawLinePayload};
use std::ops::DerefMut;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()).set(WindowPlugin {
            primary_window: Some(bevy::window::Window {
                present_mode: PresentMode::AutoVsync,
                prevent_default_event_handling: true,
                resolution: WindowResolution::new(450., 500.),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(RtcClientPlugin {
            // CAREFUL: This encoding MUST match the server encoding!
            encoding: TransportEncoding::Json,
        })
        .add_client_rw_unbounded_protocol::<ChatPayload>()
        .add_client_rw_unbounded_protocol::<DrawLinePayload>()
        .insert_resource(ChatState::default())
        .insert_resource(PaintingState::default())
        .add_systems(Startup, |mut commands: Commands| {
            // Without a camera we get no clear color
            commands.spawn(Camera2dBundle::default());
        })
        .add_systems(
            Update,
            (
                print_events,
                read_chats,
                read_lines,
                send_chats,
                send_lines,
                app_ui,
            ),
        )
        .add_systems(
            OnEnter(RtcClientStatus::Establishing),
            |mut commands: Commands| commands.insert_resource(ClearColor(Color::ORANGE)),
        )
        .add_systems(
            OnEnter(RtcClientStatus::Connected),
            |mut commands: Commands| commands.insert_resource(ClearColor(Color::GREEN)),
        )
        .add_systems(
            OnEnter(RtcClientStatus::Disconnected),
            |mut commands: Commands,
             mut chat_state: ResMut<ChatState>,
             mut painting_state: ResMut<PaintingState>| {
                commands.insert_resource(ClearColor(Color::RED));
                *chat_state = ChatState::default();
                *painting_state = PaintingState::default();
            },
        )
        .run();
}

fn print_events(mut events: EventReader<RtcClientEvent>) {
    for ev in events.read() {
        match ev {
            RtcClientEvent::IdAssigned(id) => {
                info!("ID assigned: {id:?}");
            }
            RtcClientEvent::ConnectedToHost(host) => {
                // Connected to host
                info!("Connected to host ({host})");
            }
            RtcClientEvent::DisconnectedFromHost { reason } => {
                // Disconnected from host
                warn!("Disconnected from host, reason: {reason:?}");
            }
        }
    }
}

fn read_chats(mut chat_state: ResMut<ChatState>, mut chat_read: RtcClient<ChatPayload>) {
    for chat in chat_read.read() {
        chat_state.messages.insert(0, chat);
    }
}

fn send_chats(
    mut chat_state: ResMut<ChatState>,
    mut client: RtcClient<ChatPayload>,
    rtc_state: Res<RtcClientState>,
) {
    if let Some(message) = chat_state.out.take() {
        let payload = ChatPayload {
            from: rtc_state.peer_id().unwrap().to_string(),
            message,
        };
        client.reliable_to_host(payload);
    }
}

fn read_lines(
    mut painting_state: ResMut<PaintingState>,
    mut painting_read: RtcClient<DrawLinePayload>,
) {
    for draw in painting_read.read() {
        let DrawLinePayload { x1, y1, x2, y2 } = draw;
        painting_state
            .lines
            .push(vec![Pos2::new(x1, y1), Pos2::new(x2, y2)]);
    }
}

fn send_lines(mut painting_state: ResMut<PaintingState>, mut client: RtcClient<DrawLinePayload>) {
    let draws = painting_state.out.drain(..);
    for (x1, y1, x2, y2) in draws {
        let draw = DrawLinePayload { x1, y1, x2, y2 };
        client.unreliable_to_host(draw)
    }
}

#[allow(clippy::too_many_arguments)]
fn app_ui(
    state: Res<RtcClientState>,
    connection_status: Res<State<RtcClientStatus>>,
    mut contexts: EguiContexts,
    mut painting_state: ResMut<PaintingState>,
    mut connection_requests: EventWriter<RtcClientRequestEvent>,
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
            RtcClientStatus::Establishing => {
                ui.label("Connecting...");
            }
            RtcClientStatus::Disconnected => {
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
                    connection_requests.send(RtcClientRequestEvent::Connect {
                        addr: if room_url.is_empty() {
                            "ws://127.0.0.1:3536".to_string()
                        } else {
                            room_url.to_string()
                        },
                    });
                }
            }
            RtcClientStatus::Connected => {
                if ui.button("Disconnect").clicked() {
                    connection_requests.send(RtcClientRequestEvent::Disconnect);
                }
                ui.label(format!("Connected as {}", state.peer_id().unwrap()));
                ui.label(format!(
                    "Latency: {:.0?} (smoothed = {:.0?})",
                    state.latency().unwrap_or_default(),
                    state.smoothed_latency().unwrap_or_default()
                ));

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
