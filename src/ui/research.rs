use eframe::egui;

use super::theme;

pub fn show(ui: &mut egui::Ui) {
    research_header(ui);
    ui.add_space(10.0);

    ui.columns(2, |cols| {
        article_canvas(&mut cols[0]);
        context_panel(&mut cols[1]);
    });

    ui.add_space(10.0);
    ui.columns(3, |cols| {
        notes_panel(&mut cols[0]);
        downloads_panel(&mut cols[1]);
        split_view_panel(&mut cols[2]);
    });
}

fn research_header(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("RESEARCH WORKSPACE")
                        .size(18.0)
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    egui::RichText::new("Collect evidence, compare sources and keep context attached to the browsing session.")
                        .size(10.0)
                        .color(theme::MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let _ = ui.add(
                    egui::Button::new("+ New Research Tab")
                        .fill(egui::Color32::from_rgb(25, 72, 160)),
                );
            });
        });
    });
}

fn article_canvas(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(350.0);
        ui.label(
            egui::RichText::new("SCIENCE · RESEARCH BRIEF")
                .size(9.0)
                .strong()
                .color(theme::CYAN),
        );
        ui.label(
            egui::RichText::new("The Next Era of Space Technology")
                .size(24.0)
                .strong()
                .color(theme::TEXT),
        );
        ui.label(
            egui::RichText::new(
                "A workspace mock article showing how CherryBrowser can keep browsing, notes, saved evidence and AI assistance visible without turning the browser into a dashboard landfill.",
            )
            .size(11.0)
            .color(theme::MUTED),
        );
        ui.add_space(10.0);

        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 180.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 7.0, egui::Color32::from_rgb(4, 16, 39));

        let earth = egui::pos2(rect.left() + rect.width() * 0.36, rect.bottom() + 18.0);
        let radius = 150.0_f32.min(rect.width() * 0.31);
        painter.circle_filled(earth, radius, egui::Color32::from_rgb(5, 33, 74));
        painter.circle_stroke(earth, radius, egui::Stroke::new(2.0, theme::BLUE));
        for i in 0..7 {
            let y = earth.y - radius + 18.0 + i as f32 * 22.0;
            let half = (radius * 0.84 - i as f32 * 3.0).max(30.0);
            painter.line_segment(
                [egui::pos2(earth.x - half, y), egui::pos2(earth.x + half, y)],
                egui::Stroke::new(0.8, egui::Color32::from_rgba_unmultiplied(75, 224, 255, 80)),
            );
        }

        let station_x = rect.right() - 86.0;
        painter.line_segment(
            [egui::pos2(station_x, rect.top() + 34.0), egui::pos2(station_x, rect.bottom() - 34.0)],
            egui::Stroke::new(5.0, theme::MUTED),
        );
        painter.line_segment(
            [egui::pos2(station_x - 58.0, rect.center().y), egui::pos2(station_x + 42.0, rect.center().y)],
            egui::Stroke::new(4.0, theme::BLUE),
        );
        painter.circle_filled(egui::pos2(station_x, rect.center().y), 9.0, theme::CYAN);

        painter.text(
            egui::pos2(rect.left() + 12.0, rect.top() + 12.0),
            egui::Align2::LEFT_TOP,
            "OPEN KNOWLEDGE · CONNECTED CONTEXT",
            egui::FontId::monospace(9.0),
            theme::MUTED,
        );

        ui.add_space(10.0);
        ui.columns(3, |cols| {
            insight_card(&mut cols[0], "01", "Reusable systems", "Falling launch cost changes what small teams can build.", theme::BLUE);
            insight_card(&mut cols[1], "02", "Lunar infrastructure", "Permanent systems shift exploration toward operations.", theme::VIOLET);
            insight_card(&mut cols[2], "03", "Open collaboration", "Shared standards reduce friction across research groups.", theme::CYAN);
        });
    });
}

fn context_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(350.0);
        theme::section_title(ui, "Research Context", "AI summary + evidence queue");
        ui.add_space(8.0);

        summary(ui, "Summary", "Reusable launch systems, lunar infrastructure and international standards are converging into a more accessible space economy.", theme::BLUE);
        summary(ui, "Evidence", "3 saved passages · 2 related pages · 1 downloaded report", theme::CYAN);
        summary(ui, "Questions", "What changes when launch cost drops another order of magnitude?", theme::VIOLET);

        ui.add_space(8.0);
        ui.label(egui::RichText::new("ACTIONS").size(9.0).strong().color(theme::MUTED));
        for action in ["Summarize page", "Compare sources", "Find contradictions", "Pin to notes", "Export briefing"] {
            let _ = ui.add_sized(
                [ui.available_width(), 28.0],
                egui::Button::new(action).fill(egui::Color32::from_rgb(10, 25, 53)),
            );
        }
    });
}

fn notes_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(155.0);
        theme::section_title(ui, "Notes", "Attached to this workspace");
        ui.add_space(6.0);
        row(ui, "Space Tech Key Takeaways", "just now", theme::BLUE);
        row(ui, "Clean Energy Research", "2h ago", theme::CYAN);
        row(ui, "AI Education Ideas", "5h ago", theme::VIOLET);
    });
}

fn downloads_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(155.0);
        theme::section_title(ui, "Downloads", "Research files");
        ui.add_space(6.0);
        row(ui, "space_innovation_report.pdf", "2.4 MB", theme::BLUE);
        row(ui, "lunar_base_concept.jpg", "1.8 MB", theme::VIOLET);
        row(ui, "global_energy_data.csv", "952 KB", theme::CYAN);
    });
}

fn split_view_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(155.0);
        theme::section_title(ui, "Split View", "Compare without tab ping-pong");
        ui.add_space(8.0);
        ui.columns(2, |cols| {
            mini_page(&mut cols[0], "Space Technology", theme::BLUE);
            mini_page(&mut cols[1], "Clean Energy", theme::CYAN);
        });
    });
}

fn insight_card(ui: &mut egui::Ui, number: &str, title: &str, body: &str, accent: egui::Color32) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(8, 19, 41))
        .stroke(egui::Stroke::new(1.0, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(number).size(9.0).strong().color(accent));
            ui.label(
                egui::RichText::new(title)
                    .size(11.0)
                    .strong()
                    .color(theme::TEXT),
            );
            ui.label(egui::RichText::new(body).size(9.0).color(theme::MUTED));
        });
}

fn summary(ui: &mut egui::Ui, title: &str, body: &str, accent: egui::Color32) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(8, 19, 41))
        .stroke(egui::Stroke::new(1.0, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).size(9.0).strong().color(accent));
            ui.label(egui::RichText::new(body).size(10.0).color(theme::TEXT));
        });
    ui.add_space(6.0);
}

fn row(ui: &mut egui::Ui, title: &str, meta: &str, accent: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("●").color(accent));
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).size(9.5).color(theme::TEXT));
            ui.label(egui::RichText::new(meta).size(8.0).color(theme::MUTED));
        });
    });
    ui.add_space(4.0);
}

fn mini_page(ui: &mut egui::Ui, title: &str, accent: egui::Color32) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 70.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(5, 15, 34));
    painter.rect_filled(
        egui::Rect::from_min_max(rect.left_top(), egui::pos2(rect.right(), rect.top() + 8.0)),
        4.0,
        accent,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(9.0),
        theme::TEXT,
    );
}
