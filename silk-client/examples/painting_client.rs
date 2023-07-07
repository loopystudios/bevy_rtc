use bevy::{log::LogPlugin, prelude::*, window::WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use painting::PaintingState;
use silk_client::{
    events::ConnectionRequest, AddNetworkMessageExt, NetworkReader,
    NetworkWriter, SilkClientPlugin,
};
use silk_common::{
    bevy_matchbox::prelude::*,
    demo_packets::{Chat, DrawPoint},
    events::SilkClientEvent,
    schedule::SilkSchedule,
    stage::SilkStage,
    AuthenticationRequest,
};
use std::{net::Ipv4Addr, ops::DerefMut};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
enum ConnectionState {
    #[default]
    Disconnected,
    LoggingIn,
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
                    "error,painting_client=trace,silk=trace,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
                        .into(),
                level: bevy::log::Level::DEBUG,
            })
            .set(WindowPlugin {
                primary_window: Some(bevy::window::Window {
                    fit_canvas_to_parent: true, // behave on wasm
                    resolution: WindowResolution::new(350., 650.),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugin(SilkClientPlugin)
    .add_network_message::<Chat>()
    .add_network_message::<DrawPoint>()
    .add_state::<ConnectionState>()
    .insert_resource(WorldState::default())
    .add_system(
        handle_events
            .in_base_set(SilkStage::SilkEvents)
            .in_schedule(SilkSchedule)
    )
    .add_system(login_ui)
    .add_system(chatbox_ui.in_set(OnUpdate(ConnectionState::Connected)))
    .add_system(painting_ui.in_set(OnUpdate(ConnectionState::Connected)))
    .add_system(
        on_disconnected.in_schedule(OnEnter(ConnectionState::Disconnected)),
    )    .add_system(
        on_logging_in.in_schedule(OnEnter(ConnectionState::LoggingIn)),
    )
    .add_system(
        on_connected.in_schedule(OnEnter(ConnectionState::Connected)),
    )
    .add_startup_system(setup_cam)
    .add_plugin(EguiPlugin)
    .insert_resource(MessagesState::default())
    .insert_resource(PaintingState::default())
    .run();
}

fn setup_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn on_disconnected(
    mut commands: Commands,
    mut world_state: ResMut<WorldState>,
) {
    commands.insert_resource(ClearColor(Color::RED));
    *world_state = WorldState::default();
}

fn on_logging_in(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::ORANGE));
}

fn on_connected(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::GREEN));
}

fn handle_events(
    mut app_state: ResMut<NextState<ConnectionState>>,
    mut events: EventReader<SilkClientEvent>,
    mut world_state: ResMut<WorldState>,
) {
    for ev in events.iter() {
        debug!("event: {ev:?}");
        match ev {
            SilkClientEvent::IdAssigned(id) => {
                info!("Got ID from signaling server: {id:?}");
                world_state.id.replace(*id);
            }
            SilkClientEvent::ConnectedToHost { host, username } => {
                // Connected to host
                info!("Connected to host: {host:?} as {username}");
                app_state.set(ConnectionState::Connected);
            }
            SilkClientEvent::DisconnectedFromHost { reason } => {
                // Disconnected from host
                error!("Disconnected from host, reason: {reason:?}");
                app_state.set(ConnectionState::Disconnected);
            }
        }
    }
}

fn chatbox_ui(
    mut egui_context: EguiContexts,
    world_state: Res<WorldState>,
    mut messages_state: ResMut<MessagesState>,
    mut text: Local<String>,
    mut chat_send: NetworkWriter<Chat>,
    mut chat_read: NetworkReader<Chat>,
) {
    for chat in chat_read.iter() {
        messages_state
            .messages
            .push((chat.from.clone(), chat.message.clone()));
    }

    egui::Window::new("Chat").show(egui_context.ctx_mut(), |ui| {
        ui.label("Send Message");
        ui.horizontal_wrapped(|ui| {
            ui.text_edit_singleline(text.deref_mut());
            if ui.button("Send").clicked() {
                let chat_message = Chat {
                    from: format!("{:?}", world_state.id.unwrap()),
                    message: text.to_owned(),
                };
                chat_send.reliable_to_host(chat_message);
            };
        });
        ui.label("Messages");
        messages_state.ui(ui);
    });
}

fn painting_ui(
    mut egui_context: EguiContexts,
    mut painting: ResMut<PaintingState>,
    mut draw_read: NetworkReader<DrawPoint>,
    mut draw_send: NetworkWriter<DrawPoint>,
) {
    for draw in draw_read.iter() {
        painting.lines.push(vec![
            Pos2::new(draw.x1, draw.y1),
            Pos2::new(draw.x2, draw.y2),
        ]);
    }

    egui::Window::new("Painter").show(egui_context.ctx_mut(), |ui| {
        let mut out: Option<(f32, f32, f32, f32)> = None;
        painting.ui(ui, &mut out);
        if let Some((x1, y1, x2, y2)) = out {
            let draw_point = DrawPoint { x1, y1, x2, y2 };
            draw_send.unreliable_to_host(draw_point)
        }
    });
}

fn login_ui(
    mut egui_context: EguiContexts,
    mut event_wtr: EventWriter<ConnectionRequest>,
    mut next_connection_state: ResMut<NextState<ConnectionState>>,
    world_state: Res<WorldState>,
) {
    egui::Window::new("Login").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("{:?}", world_state.id));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Connect").clicked() {
                let auth = AuthenticationRequest::Guest { username: None };
                next_connection_state.set(ConnectionState::LoggingIn);
                event_wtr.send(ConnectionRequest::Connect {
                    ip: Ipv4Addr::LOCALHOST.into(),
                    port: 3536,
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

#[derive(Resource, Default, PartialEq)]
struct MessagesState {
    messages: Vec<(String, String)>,
}

use egui::*;
impl MessagesState {
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

mod painting {
    use crate::egui;
    use bevy::prelude::Resource;
    use egui::*;

    #[derive(Resource)]
    pub struct PaintingState {
        /// in 0-1 normalized coordinates
        pub lines: Vec<Vec<Pos2>>,
        stroke: Stroke,
    }

    impl Default for PaintingState {
        fn default() -> Self {
            Self {
                lines: Default::default(),
                stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            }
        }
    }

    impl PaintingState {
        fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
            ui.horizontal(|ui| {
                egui::stroke_ui(ui, &mut self.stroke, "Stroke");
                ui.separator();
                if ui.button("Clear Painting").clicked() {
                    self.lines.clear();
                }
            })
            .response
        }

        fn ui_content(
            &mut self,
            ui: &mut Ui,
            out: &mut Option<(f32, f32, f32, f32)>,
        ) -> egui::Response {
            let (mut response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(),
                Sense::drag(),
            );

            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(
                    Pos2::ZERO,
                    response.rect.square_proportions(),
                ),
                response.rect,
            );
            let from_screen = to_screen.inverse();

            if self.lines.is_empty() {
                self.lines.push(vec![]);
            }

            let current_line = self.lines.last_mut().unwrap();

            // User has mouse down
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                if current_line.last() != Some(&canvas_pos) {
                    if let Some(last_point) = current_line.last() {
                        // Line = current_line.last() -> canvas_pos
                        let (x1, y1, x2, y2) = (
                            last_point.x,
                            last_point.y,
                            canvas_pos.x,
                            canvas_pos.y,
                        );
                        // Send to out
                        out.replace((x1, y1, x2, y2));
                    }
                    current_line.push(canvas_pos);
                    response.mark_changed();
                }
            } else if !current_line.is_empty() {
                self.lines.push(vec![]);
                response.mark_changed();
            }

            let shapes =
                self.lines
                    .iter()
                    .filter(|line| line.len() >= 2)
                    .map(|line| {
                        let points: Vec<Pos2> =
                            line.iter().map(|p| to_screen * *p).collect();
                        egui::Shape::line(points, self.stroke)
                    });

            painter.extend(shapes);

            response
        }

        pub fn ui(
            &mut self,
            ui: &mut Ui,
            out: &mut Option<(f32, f32, f32, f32)>,
        ) {
            self.ui_control(ui);
            ui.label("Paint with your mouse/touch!");
            Frame::canvas(ui.style()).show(ui, |ui| {
                self.ui_content(ui, out);
            });
        }
    }
}
