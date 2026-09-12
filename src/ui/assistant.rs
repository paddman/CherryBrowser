use eframe::egui;

use super::theme;

pub fn show(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_width(248.0);
        ui.set_max_width(272.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("CHERRY AI").strong().color(theme::TEXT));
            ui.label(egui::RichText::new("●").color(theme::GOOD));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.small_button("⋯");
            });
        });
        ui.label(
            egui::RichText::new("Always here for a brighter internet.")
                .size(10.0)
                .color(theme::MUTED),
        );
        ui.add_space(10.0);

        mascot(ui);

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            selectable_pill(ui, "Chat", true);
            selectable_pill(ui, "Search", false);
            selectable_pill(ui, "Tools", false);
        });
        ui.add_space(10.0);

        action_row(ui, "Summarize this page", "⌁");
        action_row(ui, "Explain something", "?");
        action_row(ui, "Find related content", "⌕");
        action_row(ui, "Translate / Rewrite", "文");
        action_row(ui, "Help with research", "✦");
        action_row(ui, "Open in workspace", "＋");

        ui.add_space(10.0);
        let mut prompt = String::new();
        ui.add_sized(
            [ui.available_width(), 32.0],
            egui::TextEdit::singleline(&mut prompt).hint_text("Ask Cherry anything…"),
        );
        ui.add_space(7.0);
        ui.label(
            egui::RichText::new("Browse smarter · Think farther · Together")
                .size(9.0)
                .color(theme::MUTED),
        );
    });
}

fn mascot(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 158.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(7, 24, 57));
    let center = egui::pos2(rect.center().x, rect.center().y + 7.0);

    // Futuristic halo / data orbit.
    painter.circle_stroke(center, 62.0, egui::Stroke::new(1.0, theme::BLUE));
    painter.circle_stroke(
        center,
        49.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(54, 82, 152)),
    );
    painter.line_segment(
        [
            egui::pos2(rect.left() + 14.0, rect.bottom() - 23.0),
            egui::pos2(rect.right() - 14.0, rect.top() + 25.0),
        ],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(75, 224, 255, 70)),
    );

    // Cherry avatar built as native vectors so the shell ships without raster baggage.
    let face = egui::pos2(center.x, center.y - 9.0);
    painter.circle_filled(face, 31.0, egui::Color32::from_rgb(245, 214, 207));
    painter.circle_filled(
        egui::pos2(face.x - 20.0, face.y - 13.0),
        20.0,
        egui::Color32::from_rgb(31, 80, 187),
    );
    painter.circle_filled(
        egui::pos2(face.x + 17.0, face.y - 15.0),
        19.0,
        egui::Color32::from_rgb(26, 65, 166),
    );
    painter.circle_filled(
        egui::pos2(face.x - 2.0, face.y - 23.0),
        23.0,
        egui::Color32::from_rgb(39, 102, 230),
    );
    painter.circle_filled(egui::pos2(face.x - 11.0, face.y + 2.0), 3.3, theme::BLUE);
    painter.circle_filled(egui::pos2(face.x + 12.0, face.y + 2.0), 3.3, theme::BLUE);
    painter.line_segment(
        [
            egui::pos2(face.x - 8.0, face.y + 14.0),
            egui::pos2(face.x + 8.0, face.y + 14.0),
        ],
        egui::Stroke::new(1.6, egui::Color32::from_rgb(180, 80, 94)),
    );

    let jacket = egui::Rect::from_center_size(
        egui::pos2(center.x, rect.bottom() - 25.0),
        egui::vec2(102.0, 46.0),
    );
    painter.rect_filled(jacket, 16.0, egui::Color32::from_rgb(226, 237, 255));
    painter.line_segment(
        [
            egui::pos2(jacket.left() + 16.0, jacket.top() + 4.0),
            egui::pos2(jacket.center().x, jacket.bottom() - 4.0),
        ],
        egui::Stroke::new(4.0, theme::BLUE),
    );
    painter.line_segment(
        [
            egui::pos2(jacket.right() - 16.0, jacket.top() + 4.0),
            egui::pos2(jacket.center().x, jacket.bottom() - 4.0),
        ],
        egui::Stroke::new(4.0, theme::VIOLET),
    );

    painter.text(
        egui::pos2(rect.left() + 12.0, rect.top() + 10.0),
        egui::Align2::LEFT_TOP,
        "HI, I'M CHERRY",
        egui::FontId::monospace(10.0),
        theme::CYAN,
    );
    painter.text(
        egui::pos2(rect.right() - 12.0, rect.bottom() - 10.0),
        egui::Align2::RIGHT_BOTTOM,
        "AI BROWSING COMPANION",
        egui::FontId::monospace(9.0),
        theme::MUTED,
    );
}

fn selectable_pill(ui: &mut egui::Ui, text: &str, active: bool) {
    let fill = if active {
        egui::Color32::from_rgb(27, 74, 166)
    } else {
        egui::Color32::from_rgb(10, 23, 49)
    };
    ui.add(egui::Button::new(text).fill(fill));
}

fn action_row(ui: &mut egui::Ui, label: &str, icon: &str) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(9, 21, 46))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(28, 62, 121)))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).color(theme::CYAN));
                ui.label(egui::RichText::new(label).size(11.0).color(theme::TEXT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("›").color(theme::MUTED));
                });
            });
        });
    ui.add_space(5.0);
}
