use eframe::egui;

use super::{ShellPage, theme};

pub fn show(ui: &mut egui::Ui, page: &mut ShellPage) {
    theme::card().show(ui, |ui| {
        ui.set_min_width(188.0);
        ui.set_max_width(208.0);

        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(21, 54, 118));
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 7.0, rect.center().y),
                    egui::pos2(rect.center().x, rect.top() + 7.0),
                ],
                egui::Stroke::new(3.0, theme::CYAN),
            );
            painter.line_segment(
                [
                    egui::pos2(rect.center().x, rect.top() + 7.0),
                    egui::pos2(rect.right() - 6.0, rect.center().y),
                ],
                egui::Stroke::new(3.0, theme::BLUE),
            );
            painter.line_segment(
                [
                    egui::pos2(rect.right() - 6.0, rect.center().y),
                    egui::pos2(rect.center().x, rect.bottom() - 6.0),
                ],
                egui::Stroke::new(3.0, theme::VIOLET),
            );
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("CHERRYBROWSER")
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    egui::RichText::new("A MORE OPEN TOMORROW")
                        .size(9.0)
                        .color(theme::MUTED),
                );
            });
        });

        ui.add_space(14.0);
        if nav_row(ui, "⌂", "New Tab", *page == ShellPage::NewTab) {
            *page = ShellPage::NewTab;
        }
        let _ = nav_row(ui, "✦", "AI Assistant", false);
        if nav_row(ui, "▣", "Research Workspace", *page == ShellPage::Research) {
            *page = ShellPage::Research;
        }
        let _ = nav_row(ui, "☆", "Bookmarks", false);
        let _ = nav_row(ui, "◷", "History", false);
        let _ = nav_row(ui, "⇩", "Downloads", false);
        let _ = nav_row(ui, "◎", "Security Insights", false);
        if nav_row(ui, "◇", "Themes & Settings", *page == ShellPage::Settings) {
            *page = ShellPage::Settings;
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("WORKSPACES")
                    .size(10.0)
                    .strong()
                    .color(theme::MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let _ = ui.small_button("+");
            });
        });
        ui.add_space(5.0);
        workspace_row(ui, "Personal", "12 tabs", true, theme::BLUE);
        workspace_row(ui, "Research", "6 tabs", false, theme::CYAN);
        workspace_row(ui, "Work", "9 tabs", false, theme::VIOLET);
        workspace_row(
            ui,
            "Creative",
            "4 tabs",
            false,
            egui::Color32::from_rgb(255, 91, 177),
        );

        ui.add_space(14.0);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 92.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 7.0, egui::Color32::from_rgb(5, 20, 47));
        for i in 0..10 {
            let x = rect.left() + i as f32 * rect.width() / 10.0;
            let h = 14.0 + ((i * 29) % 52) as f32;
            let tower = egui::Rect::from_min_max(
                egui::pos2(x + 2.0, rect.bottom() - h - 8.0),
                egui::pos2(
                    (x + rect.width() / 12.0).min(rect.right()),
                    rect.bottom() - 8.0,
                ),
            );
            painter.rect_filled(tower, 1.0, egui::Color32::from_rgb(12, 44, 82));
            painter.line_segment(
                [tower.left_top(), tower.right_top()],
                egui::Stroke::new(1.0, theme::BLUE),
            );
        }
        painter.text(
            egui::pos2(rect.left() + 9.0, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "BROWSE · CREATE\nCONNECT · TOGETHER",
            egui::FontId::monospace(10.0),
            theme::TEXT,
        );
    });
}

fn nav_row(ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> bool {
    let fill = if active {
        egui::Color32::from_rgb(26, 65, 139)
    } else {
        egui::Color32::from_rgb(7, 15, 34)
    };
    ui.add_sized(
        [ui.available_width(), 28.0],
        egui::Button::new(format!("{icon}   {label}"))
            .fill(fill)
            .stroke(egui::Stroke::new(
                1.0,
                if active { theme::BLUE } else { theme::BORDER },
            )),
    )
    .clicked()
}

fn workspace_row(ui: &mut egui::Ui, name: &str, tabs: &str, active: bool, accent: egui::Color32) {
    let fill = if active {
        egui::Color32::from_rgb(15, 40, 86)
    } else {
        egui::Color32::TRANSPARENT
    };
    egui::Frame::default()
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(7, 5))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("●").color(accent));
                ui.label(egui::RichText::new(name).color(theme::TEXT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(tabs).size(9.0).color(theme::MUTED));
                });
            });
        });
}
