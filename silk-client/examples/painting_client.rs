use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_matchbox::{matchbox_socket::Packet, prelude::*};
use painting::PaintingState;
use silk_client::{
    events::{SilkSendEvent, SilkSocketEvent},
    ConnectionRequest, SilkClientPlugin,
};
use silk_common::demo_packets::Payload;
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
                    "error,painting_client=debug,wgpu_core=warn,wgpu_hal=warn,matchbox_socket=warn"
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
    .insert_resource(MessagesState::default())
    .insert_resource(PaintingState::default())
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
    mut messages_state: ResMut<MessagesState>,
    mut painting_state: ResMut<PaintingState>,
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
                let packet: Packet = data.clone();
                let protocol_message = Payload::from(packet.clone());
                match protocol_message {
                    Payload::Chat { from, message } => {
                        let peer = *peer;
                        info!("{peer:?}: {}", message);
                        messages_state.messages.push((from, message));
                    }
                    Payload::DrawPoint { x1, y1, x2, y2 } => {
                        info!(
                            "{peer:?}: Draw from {:?} to {:?}",
                            (x1, y1),
                            (x2, y2)
                        );
                        painting_state
                            .lines
                            .push(vec![Pos2::new(x1, y1), Pos2::new(x2, y2)]);
                    }
                }
            }
        }
    }
}

fn ui_example_system(
    mut egui_context: ResMut<EguiContext>,
    mut event_wtr: EventWriter<ConnectionRequest>,
    world_state: Res<WorldState>,
    mut silk_event_wtr: EventWriter<SilkSendEvent>,
    mut messages_state: ResMut<MessagesState>,
    mut painting: ResMut<PaintingState>,
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
                let payload = Payload::Chat {
                    from: world_state.id.unwrap(),
                    message: text.to_owned(),
                };
                silk_event_wtr
                    .send(SilkSendEvent::ReliableSend(payload.into()));
            };
        });
        ui.label("Messages");
        messages_state.ui(ui);
    });
    egui::Window::new("Painter").show(egui_context.ctx_mut(), |ui| {
        let mut out: Option<(f32, f32, f32, f32)> = None;
        painting.ui(ui, &mut out);
        if let Some((x1, y1, x2, y2)) = out {
            let payload = Payload::DrawPoint { x1, y1, x2, y2 };
            info!("Sending Draw from {:?} to {:?}", (x1, y1), (x2, y2));
            silk_event_wtr.send(SilkSendEvent::ReliableSend(payload.into()));
        }
    });
}

#[derive(Resource, Default, PartialEq)]
struct MessagesState {
    messages: Vec<(PeerId, String)>,
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
