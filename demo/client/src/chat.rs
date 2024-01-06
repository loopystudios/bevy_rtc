use bevy::ecs::system::Resource;
use bevy_egui::egui::{self, ScrollArea, Ui};
use protocol::ChatPayload;

#[derive(Resource, Default)]
pub struct ChatState {
    pub messages: Vec<ChatPayload>,
    /// A potential message going out
    pub out: Option<String>,
}

impl ChatState {
    pub fn ui(&mut self, ui: &mut Ui) {
        let text_style = egui::TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        ScrollArea::vertical().stick_to_bottom(true).show_rows(
            ui,
            row_height,
            self.messages.len(),
            |ui, items| {
                for i in items {
                    let ChatPayload { from, message } = &self.messages[i];
                    let text = format!("<-- {from}: {message}");
                    ui.label(text);
                }
            },
        );
        ui.ctx().request_repaint();
    }
}
