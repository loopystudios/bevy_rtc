use crate::egui;
use bevy::prelude::Resource;
use egui::*;

#[derive(Resource)]
pub struct PaintingState {
    /// In 0-1 normalized coordinates
    pub lines: Vec<Vec<Pos2>>,
    /// The paint brush stroke
    stroke: Stroke,

    /// Potential line messages going out
    pub out: Vec<(f32, f32, f32, f32)>,
}

impl Default for PaintingState {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            out: Vec::new(),
        }
    }
}

impl PaintingState {
    pub fn ui(&mut self, ui: &mut Ui, out: &mut Option<(f32, f32, f32, f32)>) {
        self.ui_controls(ui);
        ui.label("Paint with your mouse/touch!");
        Frame::canvas(ui.style()).show(ui, |ui| {
            self.ui_contents(ui, out);
        });
    }

    fn ui_controls(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    fn ui_contents(
        &mut self,
        ui: &mut Ui,
        out: &mut Option<(f32, f32, f32, f32)>,
    ) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
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
                    let (x1, y1, x2, y2) = (last_point.x, last_point.y, canvas_pos.x, canvas_pos.y);
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

        let shapes = self
            .lines
            .iter()
            .filter(|line| line.len() >= 2)
            .map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });

        painter.extend(shapes);

        response
    }
}
