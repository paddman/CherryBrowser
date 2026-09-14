use eframe::egui;

use super::{Action, ShellPage, model::Workspace, theme};

pub fn show(ui: &mut egui::Ui, page: &mut ShellPage, workspace: &Workspace) -> Option<Action> {
    let mut action = None;
    theme::card().show(ui, |ui| {
        ui.label(
            egui::RichText::new("YOUR SPACE")
                .size(11.0)
                .strong()
                .color(theme::BLUE),
        );
        ui.add_space(14.0);
        for section in ShellPage::ALL {
            let label = match section {
                ShellPage::Bookmarks => format!("Bookmarks  ·  {}", workspace.data.bookmarks.len()),
                _ => section.label().to_string(),
            };
            let fill = if *page == section {
                theme::PANEL_ALT
            } else {
                theme::PANEL
            };
            if ui
                .add_sized(
                    [ui.available_width(), 38.0],
                    egui::Button::new(label).fill(fill),
                )
                .clicked()
            {
                *page = section;
            }
            ui.add_space(5.0);
        }
        ui.add_space(18.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("INDEPENDENT BY DESIGN")
                .size(10.0)
                .color(theme::VIOLET),
        );
        ui.label(
            egui::RichText::new("Your workspace. Cherry's own Rust engine.")
                .size(13.0)
                .color(theme::MUTED),
        );
        ui.add_space(12.0);
        if ui
            .add_enabled(
                workspace.can_save() && workspace.dirty,
                egui::Button::new("Save workspace"),
            )
            .clicked()
        {
            action = Some(Action::SaveWorkspace);
        }
        ui.small("Ctrl/Cmd+K  ·  Commands");
    });
    action
}

pub fn compact(ui: &mut egui::Ui, page: &mut ShellPage) {
    ui.horizontal_wrapped(|ui| {
        for section in ShellPage::ALL {
            if ui
                .selectable_label(*page == section, section.label())
                .clicked()
            {
                *page = section;
            }
        }
    });
}
