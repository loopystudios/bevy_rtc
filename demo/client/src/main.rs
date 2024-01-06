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
        .add_systems(SilkSchedule, handle_events.in_set(SilkSet::SilkEvents))
        .add_systems(Update, login_ui)
        .add_systems(SilkSchedule, read_chats.in_set(SilkSet::NetworkRead))
        .add_systems(SilkSchedule, read_lines.in_set(SilkSet::NetworkRead))
        .add_systems(SilkSchedule, send_chats.in_set(SilkSet::NetworkWrite))
        .add_systems(SilkSchedule, send_lines.in_set(SilkSet::NetworkWrite))
        .add_systems(
            Update,
            chatbox_ui.run_if(in_state(SilkConnectionState::Connected)),
        )
        .add_systems(
            Update,
            painting_ui.run_if(in_state(SilkConnectionState::Connected)),
        )
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
                info!("Got ID from signaling server: {id:?}");
            }
            SilkClientEvent::ConnectedToHost { host, username } => {
                // Connected to host
                info!("Connected to host: {host:?} as {username}");
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
        chat_state.messages.push(chat.to_owned());
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

fn chatbox_ui(
    mut egui_context: EguiContexts,
    mut chat_state: ResMut<ChatState>,
    mut text: Local<String>,
) {
    egui::Window::new("Chat").show(egui_context.ctx_mut(), |ui| {
        ui.label("Send Message");
        ui.horizontal_wrapped(|ui| {
            ui.text_edit_singleline(text.deref_mut());
            if ui.button("Send").clicked() {
                chat_state.out.replace(text.to_owned());
            };
        });
        ui.label("Messages");
        chat_state.ui(ui);
    });
}

fn painting_ui(
    mut egui_context: EguiContexts,
    mut painting: ResMut<PaintingState>,
) {
    egui::Window::new("Painter").show(egui_context.ctx_mut(), |ui| {
        let mut out: Option<(f32, f32, f32, f32)> = None;
        painting.ui(ui, &mut out);
        if let Some(draw) = out {
            painting.out.push(draw);
        }
    });
}

fn login_ui(
    mut egui_context: EguiContexts,
    mut event_wtr: EventWriter<ConnectionRequest>,
    state: Res<SilkState>,
) {
    egui::Window::new("Login").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("{:?}", state.id));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Connect").clicked() {
                let auth = AuthenticationRequest::Guest { username: None };
                event_wtr.send(ConnectionRequest::Connect {
                    addr: "ws://0.0.0.0:3536".to_string(),
                    auth,
                });
            }
            if ui.button("Disconnect").clicked() {
                event_wtr.send(ConnectionRequest::Disconnect {
                    reason: Some("User clicked disconnect".to_string()),
                });
            }
        });
    });
}
