use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
    time::Duration,
};

use eframe::egui;

use crate::{
    cancel::CancellationToken,
    css::{self, Stylesheet},
    dom::{Dom, NodeId},
    font_support, html,
    image_data::DecodedImage,
    layout::{self, LayoutDocument},
    loader::{self, LoadedDocument},
    net, renderer, text_layout, ui as browser_ui,
};
use browser_ui::{Action, ShellPage, commands::CommandPalette, model::Workspace};

const MAX_HISTORY: usize = 200;

#[derive(Debug, Clone, Copy)]
enum HistoryMode {
    Push,
    Preserve,
    Traverse(usize),
}

struct PendingNavigation {
    receiver: Receiver<Result<LoadedDocument, String>>,
    history_mode: HistoryMode,
    target: String,
    cancel: CancellationToken,
}

struct Page {
    base_url: String,
    title: String,
    status: String,
    dom: Dom,
    stylesheet: Stylesheet,
    images: HashMap<NodeId, DecodedImage>,
    textures: HashMap<NodeId, egui::TextureHandle>,
    layout: LayoutDocument,
    layout_width: f32,
}

pub struct CherryApp {
    url_input: String,
    current_url: String,
    page: Option<Page>,
    pending: Option<PendingNavigation>,
    history: Vec<String>,
    history_pos: Option<usize>,
    recent_urls: Vec<String>,
    status: String,
    hovered_href: Option<String>,
    last_error: Option<String>,
    retry_target: Option<(String, HistoryMode)>,
    native_metrics_installed: bool,
    system_font_count: usize,
    show_home: bool,
    shell_page: ShellPage,
    workspace: Workspace,
    palette: CommandPalette,
    focus_address: bool,
}

impl CherryApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        browser_ui::theme::install(&cc.egui_ctx);
        let system_font_count = font_support::install_system_fallbacks(&cc.egui_ctx).len();
        Self {
            url_input: String::new(),
            current_url: String::new(),
            page: None,
            pending: None,
            history: Vec::new(),
            history_pos: None,
            recent_urls: Vec::new(),
            status: "Cherry Engine 0.3.4 · your local workspace".to_string(),
            hovered_href: None,
            last_error: None,
            retry_target: None,
            native_metrics_installed: false,
            system_font_count,
            show_home: true,
            shell_page: ShellPage::NewTab,
            workspace: Workspace::load(),
            palette: CommandPalette::default(),
            focus_address: false,
        }
    }

    fn cancel_pending(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.cancel.cancel();
        }
    }

    fn navigate(&mut self, input: &str, history_mode: HistoryMode) {
        self.cancel_pending();
        self.hovered_href = None;
        self.retry_target = None;
        let normalized = match browser_ui::model::http_url(input) {
            Ok(url) => url,
            Err(error) => {
                self.show_home = false;
                self.last_error = Some(error.clone());
                self.status = error;
                return;
            }
        };
        self.show_home = false;
        self.url_input = normalized.clone();
        self.status = format!("Loading {normalized}");
        self.last_error = None;
        let target = normalized.clone();
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = loader::load_with_cancel(&normalized, &worker_cancel);
            let _ = sender.send(result);
        });
        self.pending = Some(PendingNavigation {
            receiver,
            history_mode,
            target,
            cancel,
        });
    }

    fn open_shell(&mut self, page: ShellPage) {
        self.cancel_pending();
        self.show_home = true;
        self.shell_page = page;
        self.hovered_href = None;
        self.last_error = None;
        self.retry_target = None;
        self.workspace.filter.clear();
        self.url_input.clear();
        self.status = format!("{} · independent Rust engine", page.label());
    }

    fn resume(&mut self) {
        self.cancel_pending();
        self.last_error = None;
        self.retry_target = None;
        self.hovered_href = None;
        if let Some(page) = self.page.as_ref() {
            self.show_home = false;
            self.url_input = self.current_url.clone();
            self.status = page.status.clone();
        } else {
            self.open_shell(ShellPage::NewTab);
        }
    }

    fn poll_navigation(&mut self) {
        let result = match self.pending.as_ref() {
            Some(pending) => match pending.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    Some(Err("network worker stopped unexpectedly".to_string()))
                }
            },
            None => None,
        };
        let Some(result) = result else {
            return;
        };
        let Some(pending) = self.pending.take() else {
            return;
        };
        match result {
            Ok(loaded) => self.install_response(loaded, pending.history_mode),
            Err(error) => {
                self.status = format!("Load failed: {error}");
                self.last_error = Some(error);
                self.retry_target = Some((pending.target, pending.history_mode));
            }
        }
    }

    fn install_response(&mut self, loaded: LoadedDocument, history_mode: HistoryMode) {
        let LoadedDocument {
            response,
            dom: loaded_dom,
            base_url,
            stylesheet_source,
            external_stylesheets,
            images,
            resource_warnings,
        } = loaded;
        let dom = if let Some(dom) = loaded_dom {
            dom
        } else if is_renderable_text(&response.content_type) {
            let escaped = escape_html(&response.body);
            html::parse(&format!("<html><body><pre>{escaped}</pre></body></html>"))
        } else {
            let message = format!(
                "CherryBrowser milestone 0.3.4 cannot render content type {} yet.",
                response.content_type
            );
            html::parse(&format!(
                "<html><body><h1>Unsupported content</h1><p>{}</p></body></html>",
                escape_html(&message)
            ))
        };
        let stylesheet = css::parse_stylesheet(&stylesheet_source);
        let title = document_title(&dom).unwrap_or_else(|| response.final_url.clone());
        let layout_width = 1000.0;
        let layout = layout::layout_document_with_images(&dom, &stylesheet, &images, layout_width);
        let image_count = images.len();
        self.current_url = response.final_url.clone();
        self.url_input = response.final_url.clone();
        self.show_home = false;
        self.status = if resource_warnings.is_empty() {
            format!(
                "HTTP {} · {} · CSS {} · IMG {} · FONT {} · Cherry Engine",
                response.status,
                response.content_type,
                external_stylesheets,
                image_count,
                self.system_font_count
            )
        } else {
            format!(
                "HTTP {} · {} · CSS {} · IMG {} · FONT {} · {} resource warning(s)",
                response.status,
                response.content_type,
                external_stylesheets,
                image_count,
                self.system_font_count,
                resource_warnings.len()
            )
        };
        self.last_error = None;
        self.retry_target = None;
        if self.recent_urls.last() != Some(&response.final_url) {
            self.recent_urls.push(response.final_url.clone());
            if self.recent_urls.len() > MAX_HISTORY {
                self.recent_urls.remove(0);
            }
        }
        match history_mode {
            HistoryMode::Push => self.push_history(response.final_url.clone()),
            HistoryMode::Preserve => {}
            HistoryMode::Traverse(pos) => self.history_pos = Some(pos),
        }
        self.page = Some(Page {
            base_url,
            title,
            status: self.status.clone(),
            dom,
            stylesheet,
            images,
            textures: HashMap::new(),
            layout,
            layout_width,
        });
    }

    fn push_history(&mut self, url: String) {
        if let Some(pos) = self.history_pos {
            self.history.truncate(pos + 1);
        }
        if self.history.last() != Some(&url) {
            self.history.push(url);
        }
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        self.history_pos = self.history.len().checked_sub(1);
    }

    fn go_back(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };
        if pos == 0 {
            return;
        }
        let next_pos = pos - 1;
        let url = self.history[next_pos].clone();
        self.navigate(&url, HistoryMode::Traverse(next_pos));
    }

    fn go_forward(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };
        if pos + 1 >= self.history.len() {
            return;
        }
        let next_pos = pos + 1;
        let url = self.history[next_pos].clone();
        self.navigate(&url, HistoryMode::Traverse(next_pos));
    }

    fn reload(&mut self) {
        if let Some((target, mode)) = self.retry_target.clone() {
            self.navigate(&target, mode);
        } else if !self.show_home && !self.current_url.is_empty() {
            let url = self.current_url.clone();
            self.navigate(&url, HistoryMode::Preserve);
        }
    }

    fn open_href(&mut self, href: &str) {
        let base = self
            .page
            .as_ref()
            .map(|page| page.base_url.as_str())
            .unwrap_or(self.current_url.as_str());
        match net::resolve_url(base, href) {
            Ok(url) => self.navigate(&url, HistoryMode::Push),
            Err(error) => {
                self.status = error.clone();
                self.last_error = Some(error);
            }
        }
    }

    fn can_go_back(&self) -> bool {
        self.history_pos.is_some_and(|pos| pos > 0)
    }

    fn can_go_forward(&self) -> bool {
        self.history_pos
            .is_some_and(|pos| pos + 1 < self.history.len())
    }

    fn page_ready(&self) -> bool {
        self.page.is_some() && self.pending.is_none() && self.last_error.is_none()
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Home => self.open_shell(ShellPage::NewTab),
            Action::Section(page) => self.open_shell(page),
            Action::Back => self.go_back(),
            Action::Forward => self.go_forward(),
            Action::Reload => self.reload(),
            Action::Stop => {
                self.resume();
                self.status =
                    "Loading stopped. Returned to the last completed page or workspace.".into();
            }
            Action::Resume => self.resume(),
            Action::Open(url) => self.navigate(&url, HistoryMode::Push),
            Action::Go => {
                match browser_ui::model::navigation_target(
                    &self.url_input,
                    self.workspace.data.search_provider,
                ) {
                    Ok(target) => self.navigate(&target, HistoryMode::Push),
                    Err(error) => {
                        self.cancel_pending();
                        self.show_home = false;
                        self.retry_target = None;
                        self.hovered_href = None;
                        self.status = error.clone();
                        self.last_error = Some(error);
                    }
                }
            }
            Action::Bookmark => {
                if self.page_ready()
                    && let Some(page) = self.page.as_ref()
                {
                    if let Err(error) = self.workspace.bookmark(&page.title, &self.current_url) {
                        self.workspace.notice = Some(error);
                    }
                    self.status = self.workspace.notice.clone().unwrap_or_default();
                }
            }
            Action::SaveWorkspace => {
                self.workspace.save();
                self.status = self.workspace.notice.clone().unwrap_or_default();
            }
            Action::ClearHistory => {
                // Do not allow an in-flight traversal to commit a now-invalid index.
                if self.pending.is_some() {
                    self.resume();
                }
                self.history.clear();
                self.history_pos = None;
                self.recent_urls.clear();
                self.status = "This window's history has been cleared.".into();
            }
        }
    }

    fn ensure_native_text_metrics(&mut self, ui: &egui::Ui) {
        if self.native_metrics_installed {
            return;
        }
        let context = ui.ctx().clone();
        text_layout::set_text_measurer(Arc::new(move |text, font_size, monospace| {
            let family = if monospace {
                egui::FontFamily::Monospace
            } else {
                egui::FontFamily::Proportional
            };
            let font = egui::FontId::new(font_size, family);
            context.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(text.to_owned(), font, egui::Color32::WHITE)
                    .size()
                    .x
            })
        }));
        self.native_metrics_installed = true;
        if let Some(page) = self.page.as_mut() {
            page.layout_width = 0.0;
        }
    }
}

impl Drop for CherryApp {
    fn drop(&mut self) {
        self.cancel_pending();
    }
}

impl eframe::App for CherryApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_navigation();
        if self.pending.is_some() {
            ctx.request_repaint_after(Duration::from_millis(40));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ensure_native_text_metrics(ui);
        let ctx = ui.ctx().clone();
        let mut action = None;
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::K)) {
            self.palette.toggle();
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::L)) {
            self.palette.open = false;
            self.focus_address = true;
        }
        if !self.palette.open {
            for (modifiers, key, next) in [
                (egui::Modifiers::COMMAND, egui::Key::R, Action::Reload),
                (egui::Modifiers::COMMAND, egui::Key::D, Action::Bookmark),
                (
                    egui::Modifiers::COMMAND,
                    egui::Key::H,
                    Action::Section(ShellPage::History),
                ),
                (
                    egui::Modifiers::COMMAND,
                    egui::Key::B,
                    Action::Section(ShellPage::Bookmarks),
                ),
                (egui::Modifiers::ALT, egui::Key::ArrowLeft, Action::Back),
                (egui::Modifiers::ALT, egui::Key::ArrowRight, Action::Forward),
                (egui::Modifiers::NONE, egui::Key::F5, Action::Reload),
            ] {
                if ctx.input_mut(|input| input.consume_key(modifiers, key)) {
                    action = Some(next);
                }
            }
            if self.pending.is_some()
                && ctx
                    .input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
            {
                action = Some(Action::Stop);
            }
        }
        let ready = self.page_ready();
        if let Some(next) = self.palette.show(
            &ctx,
            ready,
            self.workspace.can_save() && self.workspace.dirty,
        ) {
            action = Some(next);
        }

        egui::Panel::top("cherry_toolbar")
            .exact_size(94.0)
            .frame(
                egui::Frame::default()
                    .fill(browser_ui::theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("CHERRY")
                            .size(15.0)
                            .strong()
                            .color(browser_ui::theme::BLUE),
                    );
                    if ui.available_width() > 650.0 {
                        let title = if self.show_home {
                            self.shell_page.label()
                        } else {
                            self.page
                                .as_ref()
                                .map(|page| page.title.as_str())
                                .unwrap_or("Browser")
                        };
                        let title: String = title.chars().take(48).collect();
                        ui.label(egui::RichText::new(title).color(browser_ui::theme::MUTED));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Commands").on_hover_text("Ctrl/Cmd+K").clicked() {
                            self.palette.toggle();
                        }
                        if ui
                            .add_enabled(
                                self.workspace.dirty && self.workspace.can_save(),
                                egui::Button::new("Save"),
                            )
                            .on_hover_text("Save bookmarks, notes and appearance locally")
                            .clicked()
                        {
                            action = Some(Action::SaveWorkspace);
                        }
                        if ui.available_width() > 200.0
                            && ui
                                .add_enabled(ready, egui::Button::new("Bookmark page"))
                                .on_hover_text("Ctrl/Cmd+D — saves the last completed page")
                                .clicked()
                        {
                            action = Some(Action::Bookmark);
                        }
                    });
                });
                ui.horizontal(|ui| {
                    if ui.button("⌂").on_hover_text("New Tab workspace").clicked() {
                        action = Some(Action::Home);
                    }
                    if ui
                        .add_enabled(self.can_go_back(), egui::Button::new("◀"))
                        .on_hover_text("Back · Alt+Left")
                        .clicked()
                    {
                        action = Some(Action::Back);
                    }
                    if ui
                        .add_enabled(self.can_go_forward(), egui::Button::new("▶"))
                        .on_hover_text("Forward · Alt+Right")
                        .clicked()
                    {
                        action = Some(Action::Forward);
                    }
                    if self.pending.is_some() {
                        if ui
                            .button("Stop")
                            .on_hover_text("Stop loading · Esc")
                            .clicked()
                        {
                            action = Some(Action::Stop);
                        }
                    } else if ui
                        .add_enabled(
                            !self.show_home && (self.page.is_some() || self.retry_target.is_some()),
                            egui::Button::new("↻"),
                        )
                        .on_hover_text("Reload or retry · Ctrl/Cmd+R")
                        .clicked()
                    {
                        action = Some(Action::Reload);
                    }
                    let edit_width = (ui.available_width() - 46.0).max(20.0);
                    let response = ui.add_sized(
                        [edit_width, 32.0],
                        egui::TextEdit::singleline(&mut self.url_input)
                            .id(egui::Id::new("address_bar"))
                            .hint_text("URL or search · Ctrl/Cmd+L")
                            .char_limit(4096),
                    );
                    if self.focus_address {
                        response.request_focus();
                        if let Some(mut state) = egui::TextEdit::load_state(&ctx, response.id) {
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::two(
                                    egui::text::CCursor::new(0),
                                    egui::text::CCursor::new(self.url_input.chars().count()),
                                )));
                            state.store(&ctx, response.id);
                        }
                        self.focus_address = false;
                    }
                    if response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        action = Some(Action::Go);
                    }
                    if ui.button("Go").clicked() {
                        action = Some(Action::Go);
                    }
                });
            });

        let footer = self
            .hovered_href
            .clone()
            .unwrap_or_else(|| self.status.clone());
        egui::Panel::bottom("cherry_status")
            .exact_size(30.0)
            .frame(
                egui::Frame::default()
                    .fill(browser_ui::theme::BG)
                    .inner_margin(egui::Margin::symmetric(12, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.pending.is_some() {
                        ui.spinner();
                    }
                    ui.add_sized(
                        [ui.available_width(), 20.0],
                        egui::Label::new(
                            egui::RichText::new(&footer)
                                .size(12.0)
                                .color(browser_ui::theme::MUTED),
                        )
                        .truncate(),
                    )
                    .on_hover_text(&footer);
                });
            });

        let mut clicked_href = None;
        let mut hovered_href = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(browser_ui::theme::BG).inner_margin(egui::Margin::same(16)))
            .show(ui, |ui| {
                if self.show_home {
                    let current_url = self.page.as_ref().map(|_| self.current_url.as_str());
                    if let Some(next) = browser_ui::home::show(ui, &mut self.url_input, &mut self.shell_page, &mut self.workspace, &self.recent_urls, current_url) { action = Some(next); }
                } else if let Some(error) = self.last_error.as_ref() {
                    // Never conceal a failed navigation underneath a previously loaded page.
                    browser_ui::theme::card().show(ui, |ui| {
                        browser_ui::theme::section_title(ui, "This page could not be loaded", "Your last completed page is still available. No failed visit was added to history.");
                        ui.add_space(12.0);
                        ui.add(egui::Label::new(error).wrap());
                        ui.horizontal_wrapped(|ui| {
                            if ui.add_enabled(self.retry_target.is_some(), egui::Button::new("Retry request")).clicked() { action = Some(Action::Reload); }
                            if ui.add_enabled(self.page.is_some(), egui::Button::new("Return to last page")).clicked() { action = Some(Action::Resume); }
                            if ui.button("Open workspace").clicked() { action = Some(Action::Home); }
                        });
                    });
                } else if let Some(page) = self.page.as_mut() {
                    if self.pending.is_some() { ui.label("Loading a new address… The previous completed page is shown below."); }
                    let width = ui.available_width().max(320.0);
                    if (page.layout_width - width).abs() > 1.0 {
                        page.layout = layout::layout_document_with_images(&page.dom, &page.stylesheet, &page.images, width);
                        page.layout_width = width;
                    }
                    let outcome = renderer::show_document(ui, &page.layout, &page.images, &mut page.textures);
                    clicked_href = outcome.clicked_href;
                    hovered_href = outcome.hovered_href;
                } else {
                    ui.centered_and_justified(|ui| { ui.spinner(); });
                }
            });
        self.hovered_href = hovered_href;
        if let Some(action) = action {
            self.handle_action(action);
        } else if let Some(href) = clicked_href {
            self.open_href(&href);
        }
    }
}

fn document_title(dom: &Dom) -> Option<String> {
    let title = dom.find_first_tag("title")?;
    let text = dom.text_content(title);
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

fn is_renderable_text(content_type: &str) -> bool {
    let content_type = content_type.to_ascii_lowercase();
    content_type.starts_with("text/")
        || content_type.contains("html")
        || content_type.contains("xhtml")
        || content_type.contains("xml")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::{document_title, escape_html};
    use crate::html;

    #[test]
    fn extracts_title() {
        let dom = html::parse("<html><head><title> Cherry   Browser </title></head></html>");
        assert_eq!(document_title(&dom).as_deref(), Some("Cherry Browser"));
    }

    #[test]
    fn escapes_plain_text() {
        assert_eq!(escape_html("<a&b>"), "&lt;a&amp;b&gt;");
    }
}
