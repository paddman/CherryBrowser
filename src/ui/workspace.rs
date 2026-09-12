use eframe::egui;

use super::theme;

pub fn quick_links(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        quick_link(ui, "YT", "YouTube", egui::Color32::from_rgb(236, 63, 79));
        quick_link(ui, "GH", "GitHub", egui::Color32::from_rgb(220, 230, 245));
        quick_link(ui, "N", "Notion", egui::Color32::from_rgb(220, 230, 245));
        quick_link(ui, "F", "Figma", egui::Color32::from_rgb(255, 111, 89));
        quick_link(ui, "X", "X", egui::Color32::from_rgb(220, 230, 245));
        quick_link(ui, "+", "Add Site", theme::BLUE);
    });
}

pub fn feature_cards(ui: &mut egui::Ui) {
    ui.columns(3, |cols| {
        feature_card(
            &mut cols[0],
            "SPOTLIGHT",
            "A More Open Internet for Everyone",
            "Open standards. More possibilities. A kinder digital world.",
            theme::BLUE,
            0,
        );
        feature_card(
            &mut cols[1],
            "WORK TOGETHER",
            "Built Different for What's Next",
            "Research, organize and create without losing the context that matters.",
            theme::VIOLET,
            1,
        );
        feature_card(
            &mut cols[2],
            "INSIGHTS",
            "A Safer, More Connected World",
            "Private by design, transparent by default and useful without becoming creepy.",
            theme::CYAN,
            2,
        );
    });
}

pub fn bottom_dock(ui: &mut egui::Ui) {
    ui.columns(4, |cols| {
        list_panel(
            &mut cols[0],
            "Research Workspace",
            &[
                "The Next Era of Space Technology",
                "Global Clean Energy Trends",
                "AI and Education",
                "Weekly Reading List",
            ],
            theme::BLUE,
        );
        list_panel(
            &mut cols[1],
            "Saved Notes & Snippets",
            &[
                "Reusable systems reduce launch cost",
                "Global collaboration accelerates innovation",
                "Open standards expand access",
                "+ Add a new snippet",
            ],
            theme::VIOLET,
        );
        security_panel(&mut cols[2]);
        recent_panel(&mut cols[3]);
    });
}

fn quick_link(ui: &mut egui::Ui, glyph: &str, label: &str, accent: egui::Color32) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(8, 20, 44))
        .stroke(egui::Stroke::new(1.0, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::symmetric(14, 9))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(glyph).size(18.0).strong().color(accent));
                ui.label(egui::RichText::new(label).size(10.0).color(theme::TEXT));
            });
        });
}

fn feature_card(
    ui: &mut egui::Ui,
    tag: &str,
    title: &str,
    body: &str,
    accent: egui::Color32,
    variant: usize,
) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(174.0);
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 74.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(5, 20, 47));
        match variant {
            0 => {
                let c = egui::pos2(rect.center().x, rect.center().y + 4.0);
                painter.circle_stroke(c, 31.0, egui::Stroke::new(1.5, accent));
                for i in 0..7 {
                    let x = c.x - 30.0 + i as f32 * 10.0;
                    painter.line_segment(
                        [egui::pos2(x, c.y - 23.0), egui::pos2(x + 18.0, c.y + 23.0)],
                        egui::Stroke::new(
                            0.7,
                            egui::Color32::from_rgba_unmultiplied(
                                accent.r(),
                                accent.g(),
                                accent.b(),
                                90,
                            ),
                        ),
                    );
                }
            }
            1 => {
                for i in 0..8 {
                    let x = rect.left() + 8.0 + i as f32 * rect.width() / 9.0;
                    let h = 18.0 + ((i * 17) % 42) as f32;
                    let tower = egui::Rect::from_min_max(
                        egui::pos2(x, rect.bottom() - h - 7.0),
                        egui::pos2(x + 13.0, rect.bottom() - 7.0),
                    );
                    painter.rect_filled(tower, 1.0, egui::Color32::from_rgb(15, 41, 78));
                    painter.line_segment(
                        [tower.left_top(), tower.right_top()],
                        egui::Stroke::new(1.0, accent),
                    );
                }
            }
            _ => {
                let nodes = [
                    egui::pos2(rect.left() + 22.0, rect.center().y),
                    egui::pos2(rect.left() + rect.width() * 0.35, rect.top() + 18.0),
                    egui::pos2(rect.left() + rect.width() * 0.57, rect.bottom() - 18.0),
                    egui::pos2(rect.right() - 24.0, rect.top() + 24.0),
                ];
                for i in 0..nodes.len() - 1 {
                    painter.line_segment([nodes[i], nodes[i + 1]], egui::Stroke::new(1.0, accent));
                }
                for p in nodes {
                    painter.circle_filled(p, 4.0, accent);
                }
            }
        }
        ui.add_space(7.0);
        ui.label(egui::RichText::new(tag).size(9.0).strong().color(accent));
        ui.label(
            egui::RichText::new(title)
                .size(15.0)
                .strong()
                .color(theme::TEXT),
        );
        ui.label(egui::RichText::new(body).size(10.0).color(theme::MUTED));
    });
}

fn list_panel(ui: &mut egui::Ui, title: &str, rows: &[&str], accent: egui::Color32) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(184.0);
        theme::section_title(ui, title, "Context stays close to the work");
        ui.add_space(7.0);
        for (index, row) in rows.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{:02}", index + 1))
                        .size(9.0)
                        .color(accent),
                );
                ui.label(egui::RichText::new(*row).size(10.0).color(theme::TEXT));
            });
            ui.add_space(5.0);
        }
    });
}

fn security_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(184.0);
        theme::section_title(ui, "Security & Privacy", "Your browsing, your control");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("100")
                    .size(30.0)
                    .strong()
                    .color(theme::CYAN),
            );
            ui.vertical(|ui| {
                check(ui, "Tracking blockers active");
                check(ui, "Encrypted connections");
                check(ui, "Secure DNS ready");
                check(ui, "No background profiling");
            });
        });
        ui.add_space(6.0);
        ui.add(
            egui::Button::new("View security details").fill(egui::Color32::from_rgb(22, 62, 135)),
        );
    });
}

fn recent_panel(ui: &mut egui::Ui) {
    theme::card().show(ui, |ui| {
        ui.set_min_height(184.0);
        theme::section_title(ui, "Recent Sessions", "Continue where you left off");
        ui.add_space(8.0);
        session(ui, "Design Systems Research", "12 tabs · 2h ago");
        session(ui, "AI Tools Comparison", "8 tabs · 5h ago");
        session(ui, "Clean Energy Innovation", "14 tabs · yesterday");
        session(ui, "Open Source & Society", "9 tabs · Apr 24");
    });
}

fn check(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(format!("● {text}"))
            .size(9.0)
            .color(theme::GOOD),
    );
}

fn session(ui: &mut egui::Ui, title: &str, meta: &str) {
    ui.label(egui::RichText::new(title).size(10.0).color(theme::TEXT));
    ui.label(egui::RichText::new(meta).size(8.5).color(theme::MUTED));
    ui.add_space(4.0);
}
