use eframe::egui;

use super::{design_system, theme};

pub fn show(ui: &mut egui::Ui) {
    header(ui);
    ui.add_space(10.0);

    ui.columns(2, |cols| {
        appearance_panel(&mut cols[0]);
        preview_panel(&mut cols[1]);
    });

    ui.add_space(10.0);
    ui.columns(3, |cols| {
        profile_panel(&mut cols[0]);
        privacy_panel(&mut cols[1]);
        startup_panel(&mut cols[2]);
    });

    ui.add_space(10.0);
    design_system::show(ui);
}

fn header(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("SETTINGS · DESIGN SYSTEM")
                        .size(18.0)
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    egui::RichText::new("Make CherryBrowser feel like home without turning preferences into a maze of 900 mystery toggles.")
                        .size(10.0)
                        .color(theme::MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut query = String::new();
                let _ = ui.add_sized(
                    [190.0, 30.0],
                    egui::TextEdit::singleline(&mut query).hint_text("Search settings…"),
                );
            });
        });
    });
}

fn appearance_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(280.0);
        theme::section_title(ui, "Appearance", "Themes and visual tokens");
        ui.add_space(8.0);
        ui.columns(2, |cols| {
            theme_card(&mut cols[0], "Cyber Blue", theme::BLUE, true);
            theme_card(&mut cols[1], "Violet Dawn", theme::VIOLET, false);
        });
        ui.add_space(6.0);
        ui.columns(2, |cols| {
            theme_card(&mut cols[0], "Midnight", egui::Color32::from_rgb(16, 26, 53), false);
            theme_card(&mut cols[1], "Snow", egui::Color32::from_rgb(225, 234, 247), false);
        });

        ui.add_space(10.0);
        ui.label(egui::RichText::new("COLOR TOKENS").size(9.0).strong().color(theme::MUTED));
        color_token(ui, "Primary", theme::BLUE, "#3E94FF");
        color_token(ui, "Accent", theme::VIOLET, "#935CFF");
        color_token(ui, "Surface", theme::PANEL, "#070F22");
        color_token(ui, "Text", theme::TEXT, "#E5EEFF");
    });
}

fn preview_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(280.0);
        theme::section_title(ui, "Live Preview", "New Tab · Cyber Blue");
        ui.add_space(8.0);

        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 196.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 7.0, egui::Color32::from_rgb(4, 13, 31));
        painter.rect_filled(
            egui::Rect::from_min_max(rect.left_top(), egui::pos2(rect.right(), rect.top() + 26.0)),
            7.0,
            egui::Color32::from_rgb(10, 25, 55),
        );
        painter.circle_filled(egui::pos2(rect.left() + 13.0, rect.top() + 13.0), 3.5, egui::Color32::from_rgb(250, 96, 116));
        painter.circle_filled(egui::pos2(rect.left() + 25.0, rect.top() + 13.0), 3.5, egui::Color32::from_rgb(255, 193, 87));
        painter.circle_filled(egui::pos2(rect.left() + 37.0, rect.top() + 13.0), 3.5, theme::GOOD);

        let horizon = rect.bottom() - 25.0;
        for i in 0..15 {
            let x = rect.left() + 10.0 + i as f32 * rect.width() / 15.5;
            let h = 22.0 + ((i * 31) % 75) as f32;
            let tower = egui::Rect::from_min_max(
                egui::pos2(x, horizon - h),
                egui::pos2((x + 12.0).min(rect.right() - 4.0), horizon),
            );
            painter.rect_filled(tower, 1.0, egui::Color32::from_rgb(9, 33, 70));
            painter.line_segment([tower.left_top(), tower.right_top()], egui::Stroke::new(1.0, if i % 2 == 0 { theme::BLUE } else { theme::VIOLET }));
        }
        painter.text(
            egui::pos2(rect.left() + 18.0, rect.top() + 48.0),
            egui::Align2::LEFT_TOP,
            "A More Open Tomorrow",
            egui::FontId::proportional(21.0),
            theme::TEXT,
        );
        let search = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 18.0, rect.top() + 84.0),
            egui::pos2(rect.right() - 18.0, rect.top() + 116.0),
        );
        painter.rect_filled(search, 16.0, egui::Color32::from_rgb(16, 39, 82));
        painter.rect_stroke(search, 16.0, egui::Stroke::new(1.0, theme::BLUE), egui::StrokeKind::Inside);
        painter.text(
            egui::pos2(search.left() + 14.0, search.center().y),
            egui::Align2::LEFT_CENTER,
            "Search the web privately…",
            egui::FontId::proportional(10.0),
            theme::MUTED,
        );

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            let _ = ui.add(egui::Button::new("Default").fill(egui::Color32::from_rgb(27, 73, 161)));
            let _ = ui.add(egui::Button::new("Minimal"));
            let _ = ui.add(egui::Button::new("Focus"));
            let _ = ui.add(egui::Button::new("Custom"));
        });
    });
}

fn profile_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(155.0);
        theme::section_title(ui, "Profiles", "Separate work, research and personal context");
        ui.add_space(8.0);
        profile(ui, "Personal", "Synced", theme::BLUE);
        profile(ui, "Work", "Local", theme::VIOLET);
        profile(ui, "Research", "Local", theme::CYAN);
    });
}

fn privacy_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(155.0);
        theme::section_title(ui, "Privacy", "Controls should explain themselves");
        ui.add_space(8.0);
        let mut trackers = true;
        let mut cookies = false;
        let mut diagnostics = false;
        let _ = ui.checkbox(&mut trackers, "Block trackers");
        let _ = ui.checkbox(&mut cookies, "Clear cookies on exit");
        let _ = ui.checkbox(&mut diagnostics, "Send diagnostic data");
        ui.label(egui::RichText::new("No hidden behavioral profile in this concept.").size(8.5).color(theme::MUTED));
    });
}

fn startup_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(155.0);
        theme::section_title(ui, "Startup", "Choose what opens first");
        ui.add_space(8.0);
        let mut mode = 0;
        let _ = ui.radio_value(&mut mode, 0, "Continue where you left off");
        let _ = ui.radio_value(&mut mode, 1, "Open New Tab");
        let _ = ui.radio_value(&mut mode, 2, "Open a specific page");
    });
}

fn theme_card(ui: &mut egui::Ui, name: &str, accent: egui::Color32, selected: bool) {
    let stroke = if selected { 2.0 } else { 1.0 };
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(8, 18, 40))
        .stroke(egui::Stroke::new(stroke, accent))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(7))
        .show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 52.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(5, 14, 32));
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(rect.left() + 5.0, rect.bottom() - 10.0),
                    egui::pos2(rect.right() - 5.0, rect.bottom() - 5.0),
                ),
                2.0,
                accent,
            );
            painter.text(
                egui::pos2(rect.left() + 8.0, rect.top() + 8.0),
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(10.0),
                theme::TEXT,
            );
            if selected {
                painter.text(
                    egui::pos2(rect.right() - 8.0, rect.top() + 8.0),
                    egui::Align2::RIGHT_TOP,
                    "●",
                    egui::FontId::proportional(10.0),
                    accent,
                );
            }
        });
}

fn color_token(ui: &mut egui::Ui, name: &str, color: egui::Color32, hex: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(13.0, 13.0), egui::Sense::hover());
        ui.painter_at(rect).circle_filled(rect.center(), 5.0, color);
        ui.label(egui::RichText::new(name).size(9.0).color(theme::TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(hex).size(8.5).color(theme::MUTED));
        });
    });
}

fn profile(ui: &mut egui::Ui, name: &str, status: &str, accent: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("●").color(accent));
        ui.label(egui::RichText::new(name).size(10.0).color(theme::TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(status).size(8.5).color(theme::MUTED));
        });
    });
    ui.add_space(5.0);
}
