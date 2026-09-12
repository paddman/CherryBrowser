use eframe::egui;

use super::theme;

pub fn show(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(184.0);
        theme::section_title(
            ui,
            "Customize Experience",
            "Themes · Layout · Design System",
        );
        ui.add_space(9.0);

        ui.horizontal_wrapped(|ui| {
            theme_swatch(ui, "Cyber Blue", theme::BLUE, true);
            theme_swatch(ui, "Violet Dawn", theme::VIOLET, false);
            theme_swatch(ui, "Midnight", egui::Color32::from_rgb(18, 29, 58), false);
            theme_swatch(ui, "Snow", egui::Color32::from_rgb(223, 232, 246), false);
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("ACCENT").size(9.0).color(theme::MUTED));
            dot(ui, theme::BLUE);
            dot(ui, theme::CYAN);
            dot(ui, theme::VIOLET);
            dot(ui, egui::Color32::from_rgb(255, 93, 166));
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::Button::new("Primary").fill(egui::Color32::from_rgb(35, 89, 196)));
            ui.add(egui::Button::new("Secondary"));
            ui.add(egui::Button::new("Ghost"));
        });
        ui.add_space(7.0);
        ui.label(
            egui::RichText::new("8px radius · compact density · glass border · native Rust")
                .size(9.0)
                .color(theme::MUTED),
        );
    });
}

fn theme_swatch(ui: &mut egui::Ui, name: &str, accent: egui::Color32, selected: bool) {
    let frame = egui::Frame::default()
        .fill(egui::Color32::from_rgb(8, 18, 40))
        .stroke(egui::Stroke::new(if selected { 2.0 } else { 1.0 }, accent))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::same(6));
    frame.show(ui, |ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(58.0, 26.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(5, 14, 32));
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.left() + 4.0, rect.bottom() - 8.0),
                egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0),
            ),
            2.0,
            accent,
        );
        ui.label(egui::RichText::new(name).size(8.5).color(theme::TEXT));
    });
}

fn dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    ui.painter_at(rect).circle_filled(rect.center(), 5.0, color);
}
