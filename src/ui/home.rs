use eframe::egui;

use super::{
    Action, ShellPage, assistant, model::Workspace, research, settings, sidebar, theme, workspace,
};

/// Breakpoints are based on available content width, never a forced minimum canvas.
pub fn columns(width: f32, companion: bool, focus: bool) -> (f32, f32) {
    let left = if width >= 1000.0 { 188.0 } else { 0.0 };
    let right = if width >= 1320.0 && companion && !focus {
        252.0
    } else {
        0.0
    };
    (left, right)
}

pub fn show(
    ui: &mut egui::Ui,
    url_input: &mut String,
    page: &mut ShellPage,
    state: &mut Workspace,
    history: &[String],
    current_url: Option<&str>,
) -> Option<Action> {
    let mut action = None;
    egui::ScrollArea::vertical()
        .id_salt("workspace_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 10.0);
            let width = ui.available_width();
            let (left, right) = columns(width, state.data.show_companion, state.data.focus_mode);
            if left == 0.0 {
                sidebar::compact(ui, page);
                ui.add_space(12.0);
            }
            if state.dirty {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Unsaved workspace edits").color(theme::VIOLET));
                    if ui
                        .add_enabled(state.can_save(), egui::Button::new("Save workspace"))
                        .clicked()
                    {
                        action = Some(Action::SaveWorkspace);
                    }
                    ui.small("Closing without saving discards edits.");
                });
                ui.add_space(10.0);
            }
            if let Some(notice) = state.notice.as_ref() {
                ui.add(egui::Label::new(egui::RichText::new(notice).color(theme::CYAN)).wrap());
                ui.add_space(10.0);
            }
            let gaps = 16.0 * (u8::from(left > 0.0) + u8::from(right > 0.0)) as f32;
            let center = (width - left - right - gaps).max(1.0);
            ui.spacing_mut().item_spacing.x = 16.0;
            ui.horizontal_top(|ui| {
                if left > 0.0 {
                    ui.allocate_ui_with_layout(
                        egui::vec2(left, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            if let Some(next) = sidebar::show(ui, page, state) {
                                action = Some(next);
                            }
                        },
                    );
                }
                ui.allocate_ui_with_layout(
                    egui::vec2(center, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let next = match *page {
                            ShellPage::NewTab => new_tab(ui, url_input, state),
                            ShellPage::Research => research::show(ui, state, current_url),
                            ShellPage::Bookmarks => workspace::bookmarks(ui, state, true),
                            ShellPage::History => {
                                workspace::history(ui, history, &mut state.filter)
                            }
                            ShellPage::Settings => settings::show(ui, state),
                        };
                        if next.is_some() {
                            action = next;
                        }
                    },
                );
                if right > 0.0 {
                    ui.allocate_ui_with_layout(
                        egui::vec2(right, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            if let Some(next) = assistant::show(ui) {
                                action = Some(next);
                            }
                        },
                    );
                }
            });
            ui.add_space(20.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("CHERRY / BROWSE · THINK · CREATE")
                        .size(11.0)
                        .color(theme::MUTED),
                );
                ui.label(
                    egui::RichText::new("Independent engine · milestone 0.3.4")
                        .size(11.0)
                        .color(theme::VIOLET),
                );
            });
        });
    action
}

fn new_tab(ui: &mut egui::Ui, url_input: &mut String, state: &mut Workspace) -> Option<Action> {
    let mut action = None;
    if !state.data.focus_mode {
        hero(ui);
        ui.add_space(16.0);
    }
    theme::card().show(ui, |ui| {
        theme::section_title(
            ui,
            "Where will your curiosity take you?",
            "Open a URL or search the web. No request is sent until you submit.",
        );
        ui.add_space(12.0);
        let response = ui.add_sized(
            [ui.available_width(), 42.0],
            egui::TextEdit::singleline(url_input)
                .id(egui::Id::new("home_search"))
                .hint_text("Search or enter an HTTP/HTTPS address")
                .char_limit(4096)
                .margin(egui::Margin::symmetric(12, 10)),
        );
        let enter = response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
        ui.horizontal_wrapped(|ui| {
            if theme::primary_button(ui, "Search / Open") || enter {
                action = Some(Action::Go);
            }
            ui.label(
                egui::RichText::new(format!("via {}", state.data.search_provider.label()))
                    .size(12.0)
                    .color(theme::MUTED),
            );
        });
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("START EXPLORING")
                .size(11.0)
                .color(theme::BLUE),
        );
        if let Some(next) = workspace::quick_links(ui) {
            action = Some(next);
        }
    });
    ui.add_space(16.0);
    if let Some(next) = workspace::bookmarks(ui, state, false) {
        action = Some(next);
    }
    ui.add_space(16.0);
    theme::card().show(ui, |ui| {
        theme::section_title(
            ui,
            "A little more room to think",
            "Keep your sources and notes together without a wall of dashboard widgets.",
        );
        ui.horizontal_wrapped(|ui| {
            if ui.button("Open research notes").clicked() {
                action = Some(Action::Section(ShellPage::Research));
            }
            if ui.button("Appearance & capabilities").clicked() {
                action = Some(Action::Section(ShellPage::Settings));
            }
        });
    });
    action
}

fn hero(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 186.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 14.0, egui::Color32::from_rgb(17, 31, 57));
    for i in 0..12 {
        let x = rect.right() - 240.0 + i as f32 * 26.0;
        let height = 20.0 + ((i * 29) % 70) as f32;
        let tower = egui::Rect::from_min_size(
            egui::pos2(x, rect.bottom() - height),
            egui::vec2(15.0, height),
        );
        painter.rect_filled(
            tower,
            2.0,
            egui::Color32::from_rgba_unmultiplied(72, 122, 204, 35),
        );
    }
    if width >= 600.0 {
        let center = rect.right_center() - egui::vec2(104.0, 8.0);
        for radius in [42.0, 63.0, 83.0] {
            painter.circle_stroke(
                center,
                radius,
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(125, 157, 235, 70),
                ),
            );
        }
        painter.circle_filled(center + egui::vec2(42.0, 0.0), 5.0, theme::CYAN);
        painter.circle_filled(center + egui::vec2(-45.0, -44.0), 4.0, theme::VIOLET);
    }
    painter.text(
        rect.left_top() + egui::vec2(24.0, 23.0),
        egui::Align2::LEFT_TOP,
        "WELCOME TO YOUR SPACE",
        egui::FontId::monospace(10.0),
        theme::CYAN,
    );
    painter.text(
        rect.left_top() + egui::vec2(24.0, 53.0),
        egui::Align2::LEFT_TOP,
        "Make room for\nyour next idea.",
        egui::FontId::proportional(if width < 400.0 { 27.0 } else { 34.0 }),
        theme::TEXT,
    );
    painter.text(
        rect.left_bottom() + egui::vec2(24.0, -20.0),
        egui::Align2::LEFT_BOTTOM,
        "CHERRY  /  BROWSE DIFFERENT",
        egui::FontId::monospace(10.0),
        theme::VIOLET,
    );
}

#[cfg(test)]
mod tests {
    use super::columns;

    #[test]
    fn narrow_windows_do_not_reserve_side_rails() {
        for width in [320.0, 640.0, 768.0, 999.0] {
            assert_eq!(columns(width, true, false), (0.0, 0.0));
        }
    }

    #[test]
    fn companion_is_optional_and_only_on_wide_windows() {
        assert_eq!(columns(1100.0, true, false), (188.0, 0.0));
        assert_eq!(columns(1440.0, true, false), (188.0, 252.0));
        assert_eq!(columns(1440.0, false, false), (188.0, 0.0));
        assert_eq!(columns(1440.0, true, true), (188.0, 0.0));
    }
}
