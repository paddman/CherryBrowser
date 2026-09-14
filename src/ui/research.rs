use eframe::egui;

use super::{
    Action,
    model::{MAX_NOTE_CHARS, Workspace},
    theme,
};

pub fn show(ui: &mut egui::Ui, state: &mut Workspace, current_url: Option<&str>) -> Option<Action> {
    let mut action = None;
    theme::card().show(ui, |ui| {
        theme::section_title(ui, "Research workspace", "A quiet place for notes and sources. No AI request is made.");
        ui.add_space(14.0);
        if let Some(url) = current_url {
            ui.label(egui::RichText::new("LAST LOADED SOURCE").size(11.0).color(theme::CYAN));
            ui.add(egui::Label::new(url).wrap());
            ui.horizontal_wrapped(|ui| {
                if ui.button("Return to page").clicked() { action = Some(Action::Resume); }
                if ui.button("Bookmark source").clicked() { action = Some(Action::Bookmark); }
                if ui.button("Insert source URL").clicked() {
                    let insertion = format!("\nSource: {url}\n");
                    if state.data.notes.chars().count() + insertion.chars().count() <= MAX_NOTE_CHARS {
                        state.data.notes.push_str(&insertion);
                        state.dirty = true;
                    } else {
                        state.notice = Some("Notes are full. Remove some text before inserting a source.".into());
                    }
                }
            });
            ui.add_space(14.0);
        }
        let response = ui.add_sized(
            [ui.available_width(), 300.0],
            egui::TextEdit::multiline(&mut state.data.notes)
                .hint_text("Capture a thought, keep a quotation, or write your next steps…")
                .char_limit(MAX_NOTE_CHARS),
        );
        if response.changed() { state.dirty = true; }
        ui.horizontal_wrapped(|ui| {
            ui.small(format!("{} / {MAX_NOTE_CHARS} characters", state.data.notes.chars().count()));
            if ui.add_enabled(!state.data.notes.is_empty(), egui::Button::new("Copy notes")).clicked() {
                ui.ctx().copy_text(state.data.notes.clone());
                state.notice = Some("Notes copied to the system clipboard.".into());
            }
            if ui.add_enabled(state.dirty && state.can_save(), egui::Button::new("Save workspace")).clicked() {
                action = Some(Action::SaveWorkspace);
            }
        });
        ui.small("Edits stay in memory until Save workspace. Saved notes are local, unencrypted text; avoid secrets.");
    });
    action
}
