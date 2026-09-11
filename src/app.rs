use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use eframe::egui;

use crate::{
    css::{self, Stylesheet},
    dom::Dom,
    html,
    layout::{self, LayoutDocument},
    loader::{self, LoadedDocument},
    net,
    renderer,
};

#[derive(Debug, Clone, Copy)]
enum HistoryMode {
    Push,
    Preserve,
}

struct PendingNavigation {
    receiver: Receiver<Result<LoadedDocument, String>>,
    history_mode: HistoryMode,
}

struct Page {
    url: String,
    title: String,
    status_code: u16,
    dom: Dom,
    stylesheet: Stylesheet,
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
    status: String,
    hovered_href: Option<String>,
    last_error: Option<String>,
}

impl CherryApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            url_input: "https://example.com/".to_string(),
            current_url: String::new(),
            page: None,
            pending: None,
            history: Vec::new(),
            history_pos: None,
            status: "Cherry Engine 0.2".to_string(),
            hovered_href: None,
            last_error: None,
        };
        app.navigate("https://example.com/", HistoryMode::Push);
        app
    }

    fn navigate(&mut self, input: &str, history_mode: HistoryMode) {
        let normalized = match net::normalize_url(input) {
            Ok(url) => url,
            Err(error) => {
                self.last_error = Some(error.clone());
                self.status = error;
                return;
            }
        };

        self.url_input = normalized.clone();
        self.status = format!("Loading {normalized}");
        self.last_error = None;

        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = loader::load(&normalized);
            let _ = sender.send(result);
        });

        self.pending = Some(PendingNavigation {
            receiver,
            history_mode,
        });
    }

    fn poll_navigation(&mut self) {
        let result = match self.pending.as_ref() {
            Some(pending) => match pending.receiver.try_recv() {
                Ok(result) => Some((result, pending.history_mode)),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some((
                    Err("network worker stopped unexpectedly".to_string()),
                    pending.history_mode,
                )),
            },
            None => None,
        };

        let Some((result, history_mode)) = result else {
            return;
        };
        self.pending = None;

        match result {
            Ok(loaded) => self.install_response(loaded, history_mode),
            Err(error) => {
                self.status = format!("Load failed: {error}");
                self.last_error = Some(error);
            }
        }
    }

    fn install_response(&mut self, loaded: LoadedDocument, history_mode: HistoryMode) {
        let response = loaded.response;
        let dom = if is_renderable_text(&response.content_type) {
            if response.content_type.to_ascii_lowercase().contains("html") {
                html::parse(&response.body)
            } else {
                let escaped = escape_html(&response.body);
                html::parse(&format!("<html><body><pre>{escaped}</pre></body></html>"))
            }
        } else {
            let message = format!(
                "CherryBrowser milestone 0.2 cannot render content type {} yet.",
                response.content_type
            );
            html::parse(&format!(
                "<html><body><h1>Unsupported content</h1><p>{}</p></body></html>",
                escape_html(&message)
            ))
        };

        let stylesheet = css::parse_stylesheet(&loaded.stylesheet_source);
        let title = document_title(&dom).unwrap_or_else(|| response.final_url.clone());
        let layout_width = 1000.0;
        let layout = layout::layout_document(&dom, &stylesheet, layout_width);

        self.current_url = response.final_url.clone();
        self.url_input = response.final_url.clone();
        self.status = if loaded.resource_warnings.is_empty() {
            format!(
                "HTTP {} · {} · CSS {} · Cherry Engine",
                response.status, response.content_type, loaded.external_stylesheets
            )
        } else {
            format!(
                "HTTP {} · {} · CSS {} · {} resource warning(s)",
                response.status,
                response.content_type,
                loaded.external_stylesheets,
                loaded.resource_warnings.len()
            )
        };
        self.last_error = None;

        if matches!(history_mode, HistoryMode::Push) {
            self.push_history(response.final_url.clone());
        }

        self.page = Some(Page {
            url: response.final_url,
            title,
            status_code: response.status,
            dom,
            stylesheet,
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
        self.history_pos = Some(next_pos);
        self.navigate(&url, HistoryMode::Preserve);
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
        self.history_pos = Some(next_pos);
        self.navigate(&url, HistoryMode::Preserve);
    }

    fn reload(&mut self) {
        let url = if self.current_url.is_empty() {
            self.url_input.clone()
        } else {
            self.current_url.clone()
        };
        self.navigate(&url, HistoryMode::Preserve);
    }

    fn open_href(&mut self, href: &str) {
        let base = self
            .page
            .as_ref()
            .map(|page| page.url.as_str())
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
}

impl eframe::App for CherryApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_navigation();

        if self.pending.is_some() {
            ctx.request_repaint_after(Duration::from_millis(40));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        enum Action {
            Back,
            Forward,
            Reload,
            Go,
        }

        let mut action = None;

        egui::Panel::top("cherry_toolbar")
            .exact_size(48.0)
            .show(ui, |ui| {
                ui.add_space(7.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.can_go_back(), egui::Button::new("◀"))
                        .clicked()
                    {
                        action = Some(Action::Back);
                    }
                    if ui
                        .add_enabled(self.can_go_forward(), egui::Button::new("▶"))
                        .clicked()
                    {
                        action = Some(Action::Forward);
                    }
                    if ui.button("↻").clicked() {
                        action = Some(Action::Reload);
                    }

                    ui.strong("Cherry");

                    let edit_width = (ui.available_width() - 54.0).max(120.0);
                    let response = ui.add_sized(
                        [edit_width, 30.0],
                        egui::TextEdit::singleline(&mut self.url_input)
                            .hint_text("https://example.com"),
                    );
                    let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
                    if response.lost_focus() && enter {
                        action = Some(Action::Go);
                    }
                    if ui.button("Go").clicked() {
                        action = Some(Action::Go);
                    }
                });
            });

        let footer_text = self
            .hovered_href
            .clone()
            .unwrap_or_else(|| self.status.clone());

        egui::Panel::bottom("cherry_status")
            .exact_size(28.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.pending.is_some() {
                        ui.spinner();
                    }
                    ui.small(footer_text);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small("Independent Rust browser engine");
                    });
                });
            });

        let mut clicked_href = None;
        let mut hovered_href = None;

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(page) = self.page.as_mut() {
                let width = ui.available_width().max(320.0);
                if (page.layout_width - width).abs() > 1.0 {
                    page.layout = layout::layout_document(&page.dom, &page.stylesheet, width);
                    page.layout_width = width;
                }

                let outcome = renderer::show_document(ui, &page.layout);
                clicked_href = outcome.clicked_href;
                hovered_href = outcome.hovered_href;
            } else if let Some(error) = &self.last_error {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading("CherryBrowser");
                    ui.label(error);
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            }
        });

        self.hovered_href = hovered_href;

        match action {
            Some(Action::Back) => self.go_back(),
            Some(Action::Forward) => self.go_forward(),
            Some(Action::Reload) => self.reload(),
            Some(Action::Go) => {
                let target = self.url_input.clone();
                self.navigate(&target, HistoryMode::Push);
            }
            None => {}
        }

        if let Some(href) = clicked_href {
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
    use crate::html;

    use super::{document_title, escape_html};

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
