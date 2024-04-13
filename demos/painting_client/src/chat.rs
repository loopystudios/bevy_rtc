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
            self.messages.len().min(5), // Show 5 rows at most
            |ui, items| {
                for i in items.take(self.messages.len().min(5)) {
                    let ChatPayload { from, message } = &self.messages[i];
                    let text = format!("{from} says: {message}");
                    ui.label(text);
                }
            },
        );
        ui.ctx().request_repaint();
    }
}
