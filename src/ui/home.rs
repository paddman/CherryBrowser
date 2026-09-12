use eframe::egui;

use super::{assistant, design_system, sidebar, theme, workspace};

/// Render the first-party CherryBrowser start surface.
/// Returns true when the user submits the home URL field.
pub fn show(ui: &mut egui::Ui, url_input: &mut String) -> bool {
    let mut submit = false;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            tab_strip(ui);
            ui.add_space(8.0);

            let available = ui.available_width().max(760.0);
            let left_w = 206.0;
            let right_w = 270.0;
            let gap_budget = 28.0;
            let center_w = (available - left_w - right_w - gap_budget).max(420.0);

            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(left_w, 560.0),
                    egui::Layout::top_down(egui::Align::Min),
                    sidebar::show,
                );

                ui.add_space(6.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(center_w, 560.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        hero(ui);
                        ui.add_space(10.0);

                        let response = ui.add_sized(
                            [ui.available_width(), 38.0],
                            egui::TextEdit::singleline(url_input)
                                .hint_text("Search or enter URL")
                                .margin(egui::Margin::symmetric(12, 8)),
                        );
                        let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
                        if response.lost_focus() && enter {
                            submit = true;
                        }

                        ui.add_space(9.0);
                        workspace::quick_links(ui);
                        ui.add_space(10.0);
                        workspace::feature_cards(ui);
                    },
                );

                ui.add_space(6.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(right_w, 560.0),
                    egui::Layout::top_down(egui::Align::Min),
                    assistant::show,
                );
            });

            ui.add_space(12.0);
            workspace::bottom_dock(ui);
            ui.add_space(10.0);
            design_system::show(ui);
            ui.add_space(18.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("CHERRYBROWSER")
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    egui::RichText::new("— A MORE OPEN TOMORROW")
                        .size(10.0)
                        .color(theme::MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("BROWSE · CREATE · CONNECT · TOGETHER")
                            .size(9.0)
                            .color(theme::VIOLET),
                    );
                });
            });
            ui.add_space(8.0);
        });

    submit
}

fn tab_strip(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.horizontal(|ui| {
            tab(ui, "New Tab", true, theme::BLUE);
            tab(ui, "Research Workspace", false, theme::CYAN);
            tab(ui, "Design Systems", false, theme::VIOLET);
            tab(
                ui,
                "AI & Education",
                false,
                egui::Color32::from_rgb(255, 94, 171),
            );
            let _ = ui.small_button("+");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("CHERRY / NATIVE RUST UI")
                        .size(9.0)
                        .color(theme::MUTED),
                );
            });
        });
    });
}

fn tab(ui: &mut egui::Ui, text: &str, active: bool, accent: egui::Color32) {
    let fill = if active {
        egui::Color32::from_rgb(17, 46, 102)
    } else {
        egui::Color32::from_rgb(8, 18, 40)
    };
    let button = egui::Button::new(egui::RichText::new(text).size(10.0).color(if active {
        egui::Color32::WHITE
    } else {
        theme::MUTED
    }))
    .fill(fill)
    .stroke(egui::Stroke::new(if active { 1.5 } else { 1.0 }, accent));
    ui.add(button);
}

fn hero(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let height = (width * 0.34).clamp(215.0, 320.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 10.0, egui::Color32::from_rgb(4, 14, 34));

    // Holographic grid.
    let grid = egui::Color32::from_rgba_unmultiplied(56, 126, 220, 48);
    for i in 0..=12 {
        let x = rect.left() + rect.width() * i as f32 / 12.0;
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(1.0, grid),
        );
    }
    for i in 0..=7 {
        let y = rect.top() + rect.height() * i as f32 / 7.0;
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, grid),
        );
    }

    // Neon skyline with reflections.
    let horizon = rect.bottom() - 34.0;
    let building_w = (rect.width() / 32.0).max(9.0);
    for i in 0..24 {
        let x = rect.left() + 8.0 + i as f32 * rect.width() / 24.0;
        let h = 26.0 + ((i * 41) % 118) as f32;
        let tower = egui::Rect::from_min_max(
            egui::pos2(x, horizon - h),
            egui::pos2((x + building_w).min(rect.right() - 4.0), horizon),
        );
        let accent = if i % 3 == 0 {
            theme::VIOLET
        } else if i % 2 == 0 {
            theme::CYAN
        } else {
            theme::BLUE
        };
        painter.rect_filled(tower, 1.0, egui::Color32::from_rgb(7, 27, 59));
        painter.line_segment(
            [tower.left_top(), tower.right_top()],
            egui::Stroke::new(1.2, accent),
        );
        painter.line_segment(
            [
                egui::pos2(tower.center().x, horizon + 3.0),
                egui::pos2(tower.center().x, rect.bottom() - 5.0),
            ],
            egui::Stroke::new(
                0.7,
                egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 80),
            ),
        );
    }

    // Network globe on the right.
    let center = egui::pos2(
        rect.right() - rect.width() * 0.19,
        rect.top() + rect.height() * 0.42,
    );
    let radius = (height * 0.25).min(width * 0.13);
    painter.circle_stroke(center, radius, egui::Stroke::new(1.5, theme::BLUE));
    painter.circle_stroke(center, radius * 0.68, egui::Stroke::new(1.0, grid));
    painter.line_segment(
        [
            egui::pos2(center.x - radius, center.y),
            egui::pos2(center.x + radius, center.y),
        ],
        egui::Stroke::new(1.0, grid),
    );
    painter.line_segment(
        [
            egui::pos2(center.x, center.y - radius),
            egui::pos2(center.x, center.y + radius),
        ],
        egui::Stroke::new(1.0, grid),
    );
    let nodes = [
        (-0.65, -0.15, theme::CYAN),
        (-0.28, -0.62, theme::VIOLET),
        (0.18, -0.35, theme::BLUE),
        (0.64, -0.08, theme::CYAN),
        (0.48, 0.48, theme::VIOLET),
        (-0.18, 0.58, theme::BLUE),
        (-0.66, 0.28, theme::CYAN),
    ];
    let mut pts = Vec::new();
    for (nx, ny, color) in nodes {
        let p = egui::pos2(center.x + nx * radius, center.y + ny * radius);
        painter.circle_filled(p, 3.6, color);
        pts.push((p, color));
    }
    for i in 0..pts.len() {
        let (a, color) = pts[i];
        let (b, _) = pts[(i + 2) % pts.len()];
        painter.line_segment(
            [a, b],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 120),
            ),
        );
    }

    painter.text(
        egui::pos2(rect.left() + 26.0, rect.top() + 24.0),
        egui::Align2::LEFT_TOP,
        "Good Morning",
        egui::FontId::proportional(13.0),
        theme::MUTED,
    );
    painter.text(
        egui::pos2(rect.left() + 26.0, rect.top() + 48.0),
        egui::Align2::LEFT_TOP,
        "Explore a More\nOpen Tomorrow",
        egui::FontId::proportional(30.0),
        theme::TEXT,
    );
    painter.text(
        egui::pos2(rect.left() + 28.0, rect.top() + 122.0),
        egui::Align2::LEFT_TOP,
        "CherryBrowser — Fast. Thoughtful. Yours.",
        egui::FontId::proportional(12.0),
        theme::CYAN,
    );
    painter.text(
        egui::pos2(rect.left() + 28.0, rect.bottom() - 31.0),
        egui::Align2::LEFT_BOTTOM,
        "PEOPLE · IDEAS · AI · A BRIGHTER INTERNET",
        egui::FontId::monospace(9.5),
        theme::MUTED,
    );
}
