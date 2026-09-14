use eframe::egui;

use super::{Action, ShellPage, theme};

pub fn show(ui: &mut egui::Ui) -> Option<Action> {
    let mut action = None;
    theme::card().show(ui, |ui| {
        ui.label(egui::RichText::new("CHERRY").size(18.0).strong().color(theme::TEXT));
        ui.label(egui::RichText::new("YOUR WORKSPACE COMPANION").size(10.0).color(theme::BLUE));
        ui.add_space(14.0);
        mascot(ui);
        ui.add_space(14.0);
        ui.label(egui::RichText::new("One idea at a time.").size(18.0).strong());
        ui.label("Collect a useful page, write a note, and keep the next step close.");
        ui.add_space(12.0);
        if ui.add_sized([ui.available_width(), 36.0], egui::Button::new("Open research notes")).clicked() { action = Some(Action::Section(ShellPage::Research)); }
        if ui.add_sized([ui.available_width(), 36.0], egui::Button::new("Find a bookmark")).clicked() { action = Some(Action::Section(ShellPage::Bookmarks)); }
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(egui::RichText::new("AI NOT CONNECTED").size(11.0).color(theme::MUTED));
        ui.label("Chat, translation and AI summaries are not available yet. Nothing is sent to an AI service.");
        ui.add_space(12.0);
        ui.small("Ctrl/Cmd+K  Commands\nCtrl/Cmd+L  Address\nCtrl/Cmd+D  Bookmark page");
    });
    action
}

fn mascot(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 170.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 10.0, theme::PANEL_ALT);
    let center = rect.center() + egui::vec2(0.0, -5.0);
    painter.circle_stroke(center, 63.0, egui::Stroke::new(1.0, theme::BORDER));
    painter.circle_stroke(center, 53.0, egui::Stroke::new(1.0, theme::BLUE));
    let hair = egui::Color32::from_rgb(43, 91, 176);
    let skin = egui::Color32::from_rgb(245, 216, 205);
    // Short blue bob, white-and-blue jacket; no fruit, shield, key or padlock motifs.
    painter.rect_filled(
        egui::Rect::from_center_size(center + egui::vec2(0.0, -6.0), egui::vec2(79.0, 77.0)),
        27.0,
        hair,
    );
    let face = center + egui::vec2(0.0, -5.0);
    painter.circle_filled(face, 28.0, skin);
    painter.add(egui::Shape::convex_polygon(
        vec![
            face + egui::vec2(-34.0, -8.0),
            face + egui::vec2(-25.0, -35.0),
            face + egui::vec2(10.0, -39.0),
            face + egui::vec2(29.0, -24.0),
            face + egui::vec2(-3.0, -12.0),
            face + egui::vec2(-9.0, 0.0),
        ],
        hair,
        egui::Stroke::NONE,
    ));
    for dx in [-10.0, 11.0] {
        painter.circle_filled(face + egui::vec2(dx, 3.0), 4.0, egui::Color32::WHITE);
        painter.circle_filled(face + egui::vec2(dx, 4.0), 2.5, theme::BLUE);
    }
    painter.line_segment(
        [face + egui::vec2(-5.0, 15.0), face + egui::vec2(5.0, 15.0)],
        egui::Stroke::new(1.4, egui::Color32::from_rgb(168, 101, 109)),
    );
    let jacket =
        egui::Rect::from_center_size(center + egui::vec2(0.0, 50.0), egui::vec2(101.0, 46.0));
    painter.rect_filled(jacket, 16.0, theme::TEXT);
    painter.line_segment(
        [
            jacket.left_top() + egui::vec2(18.0, 5.0),
            jacket.center_bottom() - egui::vec2(0.0, 4.0),
        ],
        egui::Stroke::new(4.0, theme::BLUE),
    );
    painter.line_segment(
        [
            jacket.right_top() + egui::vec2(-18.0, 5.0),
            jacket.center_bottom() - egui::vec2(0.0, 4.0),
        ],
        egui::Stroke::new(4.0, theme::VIOLET),
    );
}
