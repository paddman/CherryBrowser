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
    net, renderer, text_layout,
};

#[derive(Debug, Clone, Copy)]
enum HistoryMode {
    Push,
    Preserve,
    Traverse(usize),
}

struct PendingNavigation {
    receiver: Receiver<Result<LoadedDocument, String>>,
    history_mode: HistoryMode,
    cancel: CancellationToken,
}

struct Page {
    base_url: String,
    title: String,
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
    status: String,
    hovered_href: Option<String>,
    last_error: Option<String>,
    native_metrics_installed: bool,
    system_font_count: usize,
    show_home: bool,
}

impl CherryApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_cherry_visuals(&cc.egui_ctx);
        let system_font_count = font_support::install_system_fallbacks(&cc.egui_ctx).len();
        Self {
            url_input: "https://example.com/".to_string(),
            current_url: String::new(),
            page: None,
            pending: None,
            history: Vec::new(),
            history_pos: None,
            status: "Cherry Engine 0.3.4 · CYRVOR visual shell".to_string(),
            hovered_href: None,
            last_error: None,
            native_metrics_installed: false,
            system_font_count,
            show_home: true,
        }
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

        if let Some(pending) = self.pending.take() {
            pending.cancel.cancel();
        }

        self.show_home = false;
        self.url_input = normalized.clone();
        self.status = format!("Loading {normalized}");
        self.last_error = None;

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
            cancel,
        });
    }

    fn show_home(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.cancel.cancel();
        }
        self.show_home = true;
        self.hovered_href = None;
        self.last_error = None;
        self.status = "Cherry Browser home · independent Rust engine".to_string();
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

        match history_mode {
            HistoryMode::Push => self.push_history(response.final_url.clone()),
            HistoryMode::Preserve => {}
            HistoryMode::Traverse(pos) => self.history_pos = Some(pos),
        }

        self.page = Some(Page {
            base_url,
            title,
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
        if self.show_home {
            return;
        }
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

impl eframe::App for CherryApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_navigation();

        if self.pending.is_some() {
            ctx.request_repaint_after(Duration::from_millis(40));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ensure_native_text_metrics(ui);

        enum Action {
            Home,
            Back,
            Forward,
            Reload,
            Go,
        }

        let mut action = None;

        egui::Panel::top("cherry_toolbar")
            .exact_size(56.0)
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(6, 12, 28)))
            .show(ui, |ui| {
                ui.add_space(9.0);
                ui.horizontal(|ui| {
                    if ui.button("⌂").on_hover_text("Cherry home").clicked() {
                        action = Some(Action::Home);
                    }
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

                    ui.label(
                        egui::RichText::new("CHERRY")
                            .color(egui::Color32::from_rgb(111, 184, 255))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("//")
                            .color(egui::Color32::from_rgb(122, 88, 255)),
                    );
                    ui.label(
                        egui::RichText::new("BROWSER")
                            .color(egui::Color32::from_rgb(204, 214, 255))
                            .strong(),
                    );

                    let edit_width = (ui.available_width() - 66.0).max(120.0);
                    let response = ui.add_sized(
                        [edit_width, 32.0],
                        egui::TextEdit::singleline(&mut self.url_input)
                            .hint_text("Search or enter URL"),
                    );
                    let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
                    if response.lost_focus() && enter {
                        action = Some(Action::Go);
                    }
                    if ui.button("GO").clicked() {
                        action = Some(Action::Go);
                    }
                });
            });

        let footer_text = self
            .hovered_href
            .clone()
            .unwrap_or_else(|| self.status.clone());
        let page_title = if self.show_home {
            "Cherry Browser // Home"
        } else {
            self.page
                .as_ref()
                .map(|page| page.title.as_str())
                .unwrap_or("CherryBrowser")
        };

        egui::Panel::bottom("cherry_status")
            .exact_size(30.0)
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(5, 10, 23)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.pending.is_some() {
                        ui.spinner();
                    }
                    ui.small(
                        egui::RichText::new(footer_text)
                            .color(egui::Color32::from_rgb(139, 171, 215)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small(
                            egui::RichText::new(page_title)
                                .color(egui::Color32::from_rgb(154, 127, 255)),
                        );
                    });
                });
            });

        let mut clicked_href = None;
        let mut hovered_href = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(4, 9, 21)))
            .show(ui, |ui| {
                if self.show_home {
                    show_cyber_home(ui);
                } else if let Some(page) = self.page.as_mut() {
                    let width = ui.available_width().max(320.0);
                    if (page.layout_width - width).abs() > 1.0 {
                        page.layout = layout::layout_document_with_images(
                            &page.dom,
                            &page.stylesheet,
                            &page.images,
                            width,
                        );
                        page.layout_width = width;
                    }

                    let outcome =
                        renderer::show_document(ui, &page.layout, &page.images, &mut page.textures);
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
            Some(Action::Home) => self.show_home(),
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

fn install_cherry_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(4, 9, 21);
    visuals.window_fill = egui::Color32::from_rgb(6, 12, 28);
    visuals.extreme_bg_color = egui::Color32::from_rgb(2, 7, 18);
    visuals.selection.bg_fill = egui::Color32::from_rgb(74, 99, 230);
    visuals.hyperlink_color = egui::Color32::from_rgb(88, 181, 255);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(13, 24, 49);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(27, 47, 87);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(52, 63, 132);
    ctx.set_visuals(visuals);
}

fn show_cyber_home(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let width = ui.available_width().max(320.0);
        let hero_height = (width * 0.38).clamp(240.0, 430.0);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(width, hero_height),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);

        let blue = egui::Color32::from_rgb(61, 150, 255);
        let violet = egui::Color32::from_rgb(142, 87, 255);
        let cyan = egui::Color32::from_rgb(64, 226, 255);
        let dim = egui::Color32::from_rgba_unmultiplied(60, 113, 190, 55);

        painter.rect_filled(rect, 10.0, egui::Color32::from_rgb(4, 10, 28));
        for i in 0..=14 {
            let x = rect.left() + rect.width() * i as f32 / 14.0;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, dim),
            );
        }
        for i in 0..=8 {
            let y = rect.top() + rect.height() * i as f32 / 8.0;
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, dim),
            );
        }

        let skyline_base = rect.bottom() - 28.0;
        for i in 0..22 {
            let column_w = rect.width() / 30.0;
            let x = rect.left() + 8.0 + i as f32 * rect.width() / 22.0;
            let h = 22.0 + ((i * 37) % 115) as f32;
            let building = egui::Rect::from_min_max(
                egui::pos2(x, skyline_base - h),
                egui::pos2((x + column_w).min(rect.right()), skyline_base),
            );
            painter.rect_filled(building, 1.0, egui::Color32::from_rgb(9, 27, 60));
            painter.line_segment(
                [building.left_top(), building.right_top()],
                egui::Stroke::new(1.5, if i % 2 == 0 { blue } else { violet }),
            );
        }

        let globe_center = egui::pos2(rect.right() - rect.width() * 0.25, rect.center().y - 16.0);
        let radius = (hero_height * 0.27).min(width * 0.17);
        painter.circle_stroke(globe_center, radius, egui::Stroke::new(2.0, blue));
        painter.circle_stroke(globe_center, radius * 0.66, egui::Stroke::new(1.0, dim));
        painter.line_segment(
            [
                egui::pos2(globe_center.x - radius, globe_center.y),
                egui::pos2(globe_center.x + radius, globe_center.y),
            ],
            egui::Stroke::new(1.0, dim),
        );
        painter.line_segment(
            [
                egui::pos2(globe_center.x, globe_center.y - radius),
                egui::pos2(globe_center.x, globe_center.y + radius),
            ],
            egui::Stroke::new(1.0, dim),
        );

        let nodes = [
            (-0.72, -0.15, blue),
            (-0.32, -0.58, violet),
            (0.12, -0.22, cyan),
            (0.55, -0.44, blue),
            (0.68, 0.18, violet),
            (0.20, 0.55, cyan),
            (-0.45, 0.48, blue),
            (-0.70, 0.18, violet),
        ];
        let mut points = Vec::new();
        for (nx, ny, color) in nodes {
            let point = egui::pos2(globe_center.x + nx * radius, globe_center.y + ny * radius);
            painter.circle_filled(point, 4.0, color);
            points.push((point, color));
        }
        for i in 0..points.len() {
            let (from, color) = points[i];
            let (to, _) = points[(i + 3) % points.len()];
            painter.line_segment(
                [from, to],
                egui::Stroke::new(
                    1.2,
                    egui::Color32::from_rgba_unmultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        125,
                    ),
                ),
            );
        }

        painter.text(
            egui::pos2(rect.left() + 30.0, rect.top() + 36.0),
            egui::Align2::LEFT_TOP,
            "CHERRY BROWSER",
            egui::FontId::proportional(34.0),
            egui::Color32::from_rgb(225, 238, 255),
        );
        painter.text(
            egui::pos2(rect.left() + 32.0, rect.top() + 82.0),
            egui::Align2::LEFT_TOP,
            "CYBER OPERATIONS UI  /  RUST ENGINE",
            egui::FontId::monospace(15.0),
            cyan,
        );
        painter.text(
            egui::pos2(rect.left() + 32.0, rect.top() + 112.0),
            egui::Align2::LEFT_TOP,
            "DETECT  ·  ANALYZE  ·  RENDER  ·  NAVIGATE",
            egui::FontId::monospace(13.0),
            egui::Color32::from_rgb(151, 165, 210),
        );

        let badge = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 30.0, rect.bottom() - 60.0),
            egui::vec2(238.0, 32.0),
        );
        painter.rect_filled(
            badge,
            5.0,
            egui::Color32::from_rgba_unmultiplied(38, 70, 130, 180),
        );
        painter.text(
            badge.center(),
            egui::Align2::CENTER_CENTER,
            "INDEPENDENT BROWSER CORE",
            egui::FontId::monospace(12.0),
            egui::Color32::WHITE,
        );

        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.heading("Browser Core");
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("LIVE DEVELOPMENT")
                    .color(violet)
                    .strong(),
            );
        });
        ui.label(
            egui::RichText::new(
                "CYRVOR-inspired blue/violet operations-room graphics added to the Cherry browser shell without replacing the independent page engine.",
            )
            .color(egui::Color32::from_gray(178)),
        );
        ui.add_space(12.0);

        ui.columns(3, |columns| {
            home_card(
                &mut columns[0],
                "01",
                "Rust Engine",
                "Native navigation, networking and renderer path without embedding Chromium/WebKit/Firefox.",
                blue,
            );
            home_card(
                &mut columns[1],
                "02",
                "HTML + CSS",
                "DOM, stylesheet parsing, layout and text rendering remain visible as first-class engine stages.",
                violet,
            );
            home_card(
                &mut columns[2],
                "03",
                "Threat-map Visuals",
                "A lightweight native graphic layer gives the browser a stronger cyber-operations identity.",
                cyan,
            );
        });
        ui.add_space(18.0);
    });
}

fn home_card(
    ui: &mut egui::Ui,
    number: &str,
    title: &str,
    body: &str,
    accent: egui::Color32,
) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(8, 17, 37))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_height(142.0);
            ui.label(egui::RichText::new(number).color(accent).strong());
            ui.add_space(4.0);
            ui.label(egui::RichText::new(title).size(18.0).strong());
            ui.add_space(8.0);
            ui.label(egui::RichText::new(body).color(egui::Color32::from_gray(180)));
        });
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
