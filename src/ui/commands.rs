use eframe::egui;

use super::{Action, ShellPage, theme};

#[derive(Default)]
pub struct CommandPalette {
    pub open: bool,
    query: String,
    selected: usize,
    focus_query: bool,
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.query.clear();
            self.selected = 0;
            self.focus_query = true;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, page_ready: bool, can_save: bool) -> Option<Action> {
        if !self.open {
            return None;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.open = false;
            return None;
        }
        // Consume navigation/Enter before the omnibox is drawn, so one key cannot
        // both run a command and submit a URL behind the palette.
        let (up, down, enter) = ctx.input_mut(|input| (
            input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
            input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
            input.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
        ));
        let mut open = self.open;
        let mut action = None;
        egui::Window::new("Cherry commands")
            .id(egui::Id::new("command_palette"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(440.0)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .show(ctx, |ui| {
                let response = ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Find a command…").desired_width(f32::INFINITY).char_limit(256));
                if self.focus_query {
                    response.request_focus();
                    self.focus_query = false;
                }
                if response.changed() { self.selected = 0; }
                let query = self.query.to_lowercase();
                let entries: Vec<_> = command_list(page_ready, can_save)
                    .into_iter()
                    .filter(|(label, _)| label.to_lowercase().contains(&query))
                    .collect();
                if !entries.is_empty() {
                    self.selected = self.selected.min(entries.len() - 1);
                    if down { self.selected = (self.selected + 1) % entries.len(); }
                    if up { self.selected = (self.selected + entries.len() - 1) % entries.len(); }
                }
                ui.add_space(10.0);
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for (index, (label, command)) in entries.iter().enumerate() {
                        let response = ui.add_sized([ui.available_width(), 32.0], egui::Button::new(*label).fill(if index == self.selected { theme::PANEL_ALT } else { theme::PANEL }));
                        if response.clicked() { action = Some(command.clone()); }
                        if index == self.selected && (up || down) { response.scroll_to_me(Some(egui::Align::Center)); }
                    }
                });
                if enter && let Some((_, command)) = entries.get(self.selected) { action = Some(command.clone()); }
                if entries.is_empty() { ui.label("No matching commands."); }
                ui.small("Up / Down to choose · Enter to run · Esc to close");
            });
        self.open = open && action.is_none();
        action
    }
}

fn command_list(page_ready: bool, can_save: bool) -> Vec<(&'static str, Action)> {
    let mut commands = vec![
        ("Open New Tab workspace", Action::Home),
        ("Open research notes", Action::Section(ShellPage::Research)),
        ("Manage bookmarks", Action::Section(ShellPage::Bookmarks)),
        ("Show this window's history", Action::Section(ShellPage::History)),
        ("Appearance and engine capabilities", Action::Section(ShellPage::Settings)),
    ];
    if page_ready {
        commands.push(("Return to the loaded page", Action::Resume));
        commands.push(("Bookmark the loaded page", Action::Bookmark));
    }
    if can_save { commands.push(("Save workspace locally", Action::SaveWorkspace)); }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_actions_are_not_presented_as_working_commands() {
        let commands = command_list(false, false);
        assert!(!commands.iter().any(|(_, action)| matches!(action, Action::Bookmark | Action::Resume | Action::SaveWorkspace)));
        assert_eq!(command_list(true, true).len(), commands.len() + 3);
    }
}
