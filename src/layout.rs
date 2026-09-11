use std::collections::HashMap;

use crate::{
    css::{self, Properties, Stylesheet},
    dom::{Dom, NodeId, NodeKind},
    image_data::DecodedImage,
    text_layout,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const LINK: Self = Self::rgb(25, 103, 210);
    pub const PLACEHOLDER: Self = Self::rgb(224, 228, 235);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

#[derive(Debug, Clone)]
pub enum PaintItem {
    Rect(RectItem),
    Image(ImageItem),
    Text(TextItem),
}

#[derive(Debug, Clone)]
pub struct RectItem {
    pub rect: RectF,
    pub color: Rgba,
}

#[derive(Debug, Clone)]
pub struct ImageItem {
    pub rect: RectF,
    pub node: NodeId,
}

#[derive(Debug, Clone)]
pub struct TextItem {
    pub rect: RectF,
    pub text: String,
    pub font_size: f32,
    pub color: Rgba,
    pub bold: bool,
    pub italic: bool,
    pub monospace: bool,
    pub underline: bool,
    pub href: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LayoutDocument {
    pub items: Vec<PaintItem>,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Display {
    None,
    Block,
    Inline,
}

#[derive(Debug, Clone, Copy, Default)]
struct Edges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

#[derive(Debug, Clone)]
struct ComputedStyle {
    display: Display,
    color: Rgba,
    background: Option<Rgba>,
    font_size: f32,
    line_height: f32,
    bold: bool,
    italic: bool,
    monospace: bool,
    underline: bool,
    margin: Edges,
    padding: Edges,
    width: Option<f32>,
    height: Option<f32>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            color: Rgba::BLACK,
            background: None,
            font_size: 16.0,
            line_height: 21.6,
            bold: false,
            italic: false,
            monospace: false,
            underline: false,
            margin: Edges::default(),
            padding: Edges::default(),
            width: None,
            height: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    left: f32,
    width: f32,
}

impl Bounds {
    fn right(self) -> f32 {
        self.left + self.width
    }
}

pub fn layout_document(dom: &Dom, stylesheet: &Stylesheet, viewport_width: f32) -> LayoutDocument {
    let images = HashMap::new();
    layout_document_with_images(dom, stylesheet, &images, viewport_width)
}

pub fn layout_document_with_images(
    dom: &Dom,
    stylesheet: &Stylesheet,
    images: &HashMap<NodeId, DecodedImage>,
    viewport_width: f32,
) -> LayoutDocument {
    let width = viewport_width.max(320.0);
    let mut flow = Flow::new(dom, stylesheet, images, width);
    let root_style = ComputedStyle::default();
    let inherited = Properties::new();
    flow.layout_node(
        dom.root(),
        &inherited,
        &root_style,
        None,
        Bounds { left: 0.0, width },
    );
    flow.finish_line();

    LayoutDocument {
        items: flow.items,
        width,
        height: flow.y.max(1.0) + 24.0,
    }
}

struct Flow<'a> {
    dom: &'a Dom,
    stylesheet: &'a Stylesheet,
    images: &'a HashMap<NodeId, DecodedImage>,
    items: Vec<PaintItem>,
    x: f32,
    y: f32,
    line_height: f32,
    line_left: f32,
    line_right: f32,
}

impl<'a> Flow<'a> {
    fn new(
        dom: &'a Dom,
        stylesheet: &'a Stylesheet,
        images: &'a HashMap<NodeId, DecodedImage>,
        width: f32,
    ) -> Self {
        Self {
            dom,
            stylesheet,
            images,
            items: Vec::new(),
            x: 0.0,
            y: 0.0,
            line_height: 0.0,
            line_left: 0.0,
            line_right: width,
        }
    }

    fn layout_node(
        &mut self,
        node: NodeId,
        inherited: &Properties,
        parent_style: &ComputedStyle,
        href: Option<&str>,
        bounds: Bounds,
    ) {
        let kind = self.dom.node(node).kind.clone();
        match kind {
            NodeKind::Document => {
                for child in self.dom.node(node).children.clone() {
                    self.layout_node(child, inherited, parent_style, href, bounds);
                }
            }
            NodeKind::Text(text) => {
                self.layout_text(&text, parent_style, href, bounds);
            }
            NodeKind::Element(element) => {
                let properties = css::cascade(self.dom, node, self.stylesheet, inherited);
                let style = ComputedStyle::for_element(
                    &element.tag_name,
                    &properties,
                    parent_style,
                    bounds,
                );
                if style.display == Display::None {
                    return;
                }

                let owned_href = if element.tag_name == "a" {
                    self.dom.attr(node, "href").map(ToOwned::to_owned)
                } else {
                    href.map(ToOwned::to_owned)
                };
                let current_href = owned_href.as_deref().or(href);

                if element.tag_name == "br" {
                    self.line_break(style.line_height);
                    return;
                }

                if element.tag_name == "img" {
                    self.layout_image(node, &style, bounds);
                    return;
                }

                if style.display == Display::Block {
                    self.layout_block(
                        node,
                        &properties,
                        &style,
                        current_href,
                        bounds,
                        &element.tag_name,
                    );
                } else {
                    for child in self.dom.node(node).children.clone() {
                        self.layout_node(child, &properties, &style, current_href, bounds);
                    }
                }
            }
        }
    }

    fn layout_block(
        &mut self,
        node: NodeId,
        properties: &Properties,
        style: &ComputedStyle,
        href: Option<&str>,
        parent_bounds: Bounds,
        tag: &str,
    ) {
        self.finish_line();

        self.y += style.margin.top;
        let block_left = parent_bounds.left + style.margin.left;
        let available_width =
            (parent_bounds.width - style.margin.left - style.margin.right).max(1.0);
        let block_width = style
            .width
            .unwrap_or(available_width)
            .min(available_width)
            .max(1.0);
        let block_top = self.y;

        let content_left = block_left + style.padding.left;
        let content_width = (block_width - style.padding.left - style.padding.right).max(1.0);
        let content_bounds = Bounds {
            left: content_left,
            width: content_width,
        };

        let old_left = self.line_left;
        let old_right = self.line_right;
        self.line_left = content_bounds.left;
        self.line_right = content_bounds.right();
        self.x = self.line_left;
        self.y += style.padding.top;

        if tag == "li" {
            self.layout_text("•", style, href, content_bounds);
        }

        for child in self.dom.node(node).children.clone() {
            self.layout_node(child, properties, style, href, content_bounds);
        }
        self.finish_line();
        self.y += style.padding.bottom;

        let minimum_height = style.height.unwrap_or(0.0);
        let content_height = (self.y - block_top).max(minimum_height);
        if content_height > self.y - block_top {
            self.y = block_top + content_height;
        }

        if let Some(color) = style.background {
            self.items.push(PaintItem::Rect(RectItem {
                rect: RectF {
                    x: block_left,
                    y: block_top,
                    width: block_width,
                    height: content_height,
                },
                color,
            }));
        }

        self.y += style.margin.bottom;
        self.line_left = old_left;
        self.line_right = old_right;
        self.x = parent_bounds.left;
        self.line_height = 0.0;
    }

    fn layout_text(
        &mut self,
        text: &str,
        style: &ComputedStyle,
        href: Option<&str>,
        bounds: Bounds,
    ) {
        if text.trim().is_empty() {
            return;
        }

        if self.line_left != bounds.left || (self.line_right - bounds.right()).abs() > f32::EPSILON
        {
            self.line_left = bounds.left;
            self.line_right = bounds.right();
            if self.line_height == 0.0 {
                self.x = self.line_left;
            }
        }

        let space_width = measure_text(" ", style);
        let mut pending_space = false;
        for unit in text_layout::line_break_units(text) {
            if unit.is_space {
                pending_space = self.x > self.line_left;
                continue;
            }

            let width = measure_text(&unit.text, style).max(1.0);
            let gap = if pending_space && self.x > self.line_left {
                space_width
            } else {
                0.0
            };

            if self.x + gap + width > self.line_right && self.x > self.line_left {
                self.line_break(style.line_height);
                pending_space = false;
            } else {
                self.x += gap;
                pending_space = false;
            }

            let height = style.line_height.max(style.font_size);
            self.items.push(PaintItem::Text(TextItem {
                rect: RectF {
                    x: self.x,
                    y: self.y,
                    width,
                    height,
                },
                text: unit.text,
                font_size: style.font_size,
                color: style.color,
                bold: style.bold,
                italic: style.italic,
                monospace: style.monospace,
                underline: style.underline || href.is_some(),
                href: href.map(ToOwned::to_owned),
            }));

            self.x += width;
            self.line_height = self.line_height.max(height);
        }
    }

    fn layout_image(&mut self, node: NodeId, style: &ComputedStyle, bounds: Bounds) {
        let intrinsic = self.images.get(&node);
        let explicit_width = style
            .width
            .or_else(|| self.dom.attr(node, "width").and_then(parse_plain_px));
        let explicit_height = style
            .height
            .or_else(|| self.dom.attr(node, "height").and_then(parse_plain_px));

        let width = explicit_width
            .or_else(|| intrinsic.map(|image| image.width as f32))
            .unwrap_or(240.0)
            .clamp(24.0, bounds.width);
        let height = explicit_height
            .or_else(|| {
                intrinsic.map(|image| {
                    let ratio = image.height as f32 / image.width.max(1) as f32;
                    width * ratio
                })
            })
            .unwrap_or(140.0)
            .max(24.0);

        if self.x + width > self.line_right && self.x > self.line_left {
            self.line_break(height);
        }

        let rect = RectF {
            x: self.x,
            y: self.y,
            width,
            height,
        };
        self.items.push(PaintItem::Rect(RectItem {
            rect,
            color: Rgba::PLACEHOLDER,
        }));

        if intrinsic.is_some() {
            self.items.push(PaintItem::Image(ImageItem { rect, node }));
        } else {
            let alt = self.dom.attr(node, "alt").unwrap_or("image");
            self.items.push(PaintItem::Text(TextItem {
                rect: RectF {
                    x: self.x + 8.0,
                    y: self.y + 8.0,
                    width: (width - 16.0).max(1.0),
                    height: style.line_height,
                },
                text: alt.to_string(),
                font_size: style.font_size.min(14.0),
                color: Rgba::rgb(70, 76, 86),
                bold: false,
                italic: false,
                monospace: false,
                underline: false,
                href: None,
            }));
        }

        self.x += width + 8.0;
        self.line_height = self.line_height.max(height + 8.0);
    }

    fn finish_line(&mut self) {
        if self.line_height > 0.0 {
            self.y += self.line_height;
            self.line_height = 0.0;
        }
        self.x = self.line_left;
    }

    fn line_break(&mut self, minimum_height: f32) {
        self.y += self.line_height.max(minimum_height);
        self.line_height = 0.0;
        self.x = self.line_left;
    }
}

impl ComputedStyle {
    fn for_element(
        tag: &str,
        properties: &Properties,
        parent: &ComputedStyle,
        bounds: Bounds,
    ) -> Self {
        let mut style = Self {
            display: ua_display(tag),
            color: parent.color,
            background: None,
            font_size: parent.font_size,
            line_height: parent.line_height,
            bold: parent.bold,
            italic: parent.italic,
            monospace: parent.monospace,
            underline: false,
            margin: Edges::default(),
            padding: Edges::default(),
            width: None,
            height: None,
        };

        apply_ua_style(tag, &mut style);

        if let Some(value) = properties.get("display") {
            style.display = match value.trim().to_ascii_lowercase().as_str() {
                "none" => Display::None,
                "inline" | "inline-block" => Display::Inline,
                "block" | "flex" | "grid" | "table" | "list-item" => Display::Block,
                _ => style.display,
            };
        }

        if let Some(value) = properties.get("color").and_then(|v| parse_color(v)) {
            style.color = value;
        }
        if let Some(value) = properties
            .get("background-color")
            .or_else(|| properties.get("background"))
            .and_then(|v| parse_color(v))
        {
            style.background = Some(value);
        }

        if let Some(value) = properties.get("font-size") {
            style.font_size = parse_length(value, parent.font_size, bounds.width)
                .unwrap_or(style.font_size)
                .clamp(6.0, 128.0);
        }

        if let Some(value) = properties.get("line-height") {
            style.line_height = parse_line_height(value, style.font_size)
                .unwrap_or(style.font_size * 1.35)
                .max(style.font_size);
        } else {
            style.line_height = style.line_height.max(style.font_size * 1.2);
        }

        if let Some(value) = properties.get("font-weight") {
            let normalized = value.trim().to_ascii_lowercase();
            style.bold = normalized == "bold"
                || normalized == "bolder"
                || normalized.parse::<u16>().is_ok_and(|weight| weight >= 600);
        }
        if let Some(value) = properties.get("font-style") {
            style.italic = matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "italic" | "oblique"
            );
        }
        if let Some(value) = properties.get("font-family") {
            style.monospace = value.to_ascii_lowercase().contains("monospace");
        }
        if let Some(value) = properties.get("text-decoration") {
            style.underline = value.to_ascii_lowercase().contains("underline");
        }

        if let Some(value) = properties.get("margin") {
            style.margin = parse_edges(value, style.font_size, bounds.width);
        }
        if let Some(value) = properties.get("padding") {
            style.padding = parse_edges(value, style.font_size, bounds.width);
        }

        apply_edge_override(
            properties,
            "margin-top",
            &mut style.margin.top,
            style.font_size,
            bounds.width,
        );
        apply_edge_override(
            properties,
            "margin-right",
            &mut style.margin.right,
            style.font_size,
            bounds.width,
        );
        apply_edge_override(
            properties,
            "margin-bottom",
            &mut style.margin.bottom,
            style.font_size,
            bounds.width,
        );
        apply_edge_override(
            properties,
            "margin-left",
            &mut style.margin.left,
            style.font_size,
            bounds.width,
        );
        apply_edge_override(
            properties,
            "padding-top",
            &mut style.padding.top,
            style.font_size,
            bounds.width,
        );
        apply_edge_override(
            properties,
            "padding-right",
            &mut style.padding.right,
            style.font_size,
            bounds.width,
        );
        apply_edge_override(
            properties,
            "padding-bottom",
            &mut style.padding.bottom,
            style.font_size,
            bounds.width,
        );
        apply_edge_override(
            properties,
            "padding-left",
            &mut style.padding.left,
            style.font_size,
            bounds.width,
        );

        style.width = properties
            .get("width")
            .and_then(|value| parse_length(value, style.font_size, bounds.width));
        style.height = properties
            .get("height")
            .and_then(|value| parse_length(value, style.font_size, bounds.width));

        style
    }
}

fn ua_display(tag: &str) -> Display {
    match tag {
        "head" | "script" | "style" | "title" | "meta" | "link" | "base" | "template" => {
            Display::None
        }
        "html" | "body" | "main" | "header" | "footer" | "nav" | "section" | "article"
        | "aside" | "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote"
        | "pre" | "form" | "fieldset" | "ul" | "ol" | "li" | "dl" | "dt" | "dd" | "table"
        | "thead" | "tbody" | "tfoot" | "tr" | "hr" => Display::Block,
        _ => Display::Inline,
    }
}

fn apply_ua_style(tag: &str, style: &mut ComputedStyle) {
    match tag {
        "body" => {
            style.margin = Edges {
                top: 8.0,
                right: 8.0,
                bottom: 8.0,
                left: 8.0,
            }
        }
        "p" => {
            style.margin.top = style.font_size;
            style.margin.bottom = style.font_size;
        }
        "h1" => heading(style, 2.0, 0.67),
        "h2" => heading(style, 1.5, 0.83),
        "h3" => heading(style, 1.17, 1.0),
        "h4" | "h5" | "h6" => heading(style, 1.0, 1.0),
        "a" => {
            style.color = Rgba::LINK;
            style.underline = true;
        }
        "b" | "strong" => style.bold = true,
        "i" | "em" => style.italic = true,
        "code" | "pre" | "kbd" | "samp" => style.monospace = true,
        "blockquote" => {
            style.margin.left = style.font_size * 2.0;
            style.margin.right = style.font_size * 2.0;
        }
        "ul" | "ol" => style.padding.left = style.font_size * 2.0,
        "li" => style.margin.bottom = style.font_size * 0.25,
        "hr" => {
            style.height = Some(1.0);
            style.margin.top = 8.0;
            style.margin.bottom = 8.0;
            style.background = Some(Rgba::rgb(190, 194, 201));
        }
        _ => {}
    }
}

fn heading(style: &mut ComputedStyle, scale: f32, margin_em: f32) {
    style.font_size *= scale;
    style.line_height = style.font_size * 1.2;
    style.bold = true;
    style.margin.top = style.font_size * margin_em;
    style.margin.bottom = style.font_size * margin_em;
}

fn measure_text(text: &str, style: &ComputedStyle) -> f32 {
    text_layout::estimated_text_width(text, style.font_size, style.monospace)
}

fn parse_edges(value: &str, font_size: f32, reference: f32) -> Edges {
    let values = value
        .split_whitespace()
        .filter_map(|token| parse_length(token, font_size, reference))
        .collect::<Vec<_>>();

    match values.as_slice() {
        [all] => Edges {
            top: *all,
            right: *all,
            bottom: *all,
            left: *all,
        },
        [vertical, horizontal] => Edges {
            top: *vertical,
            right: *horizontal,
            bottom: *vertical,
            left: *horizontal,
        },
        [top, horizontal, bottom] => Edges {
            top: *top,
            right: *horizontal,
            bottom: *bottom,
            left: *horizontal,
        },
        [top, right, bottom, left, ..] => Edges {
            top: *top,
            right: *right,
            bottom: *bottom,
            left: *left,
        },
        _ => Edges::default(),
    }
}

fn apply_edge_override(
    properties: &Properties,
    name: &str,
    target: &mut f32,
    font_size: f32,
    reference: f32,
) {
    if let Some(value) = properties
        .get(name)
        .and_then(|value| parse_length(value, font_size, reference))
    {
        *target = value;
    }
}

fn parse_line_height(value: &str, font_size: f32) -> Option<f32> {
    let value = value.trim();
    if let Ok(multiplier) = value.parse::<f32>() {
        return Some(multiplier * font_size);
    }
    parse_length(value, font_size, font_size)
}

fn parse_length(value: &str, font_size: f32, reference: f32) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if value == "auto" {
        return None;
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().ok();
    }
    if let Some(em) = value.strip_suffix("em") {
        return em.trim().parse::<f32>().ok().map(|v| v * font_size);
    }
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|v| v * 16.0);
    }
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f32>()
            .ok()
            .map(|v| reference * v / 100.0);
    }
    value.parse().ok()
}

fn parse_plain_px(value: &str) -> Option<f32> {
    value
        .trim()
        .trim_end_matches("px")
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
}

fn parse_color(value: &str) -> Option<Rgba> {
    let value = value.trim().to_ascii_lowercase();
    let first = value.split_whitespace().next()?;

    if first == "transparent" {
        return None;
    }

    if let Some(hex) = first.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let mut chars = hex.chars();
                let r = hex_digit(chars.next()?)? * 17;
                let g = hex_digit(chars.next()?)? * 17;
                let b = hex_digit(chars.next()?)? * 17;
                Some(Rgba::rgb(r, g, b))
            }
            6 => Some(Rgba::rgb(
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
            )),
            _ => None,
        };
    }

    if first.starts_with("rgb(") && first.ends_with(')') {
        let body = &first[4..first.len() - 1];
        let parts = body
            .split(',')
            .filter_map(|part| part.trim().parse::<u8>().ok())
            .collect::<Vec<_>>();
        if let [r, g, b] = parts.as_slice() {
            return Some(Rgba::rgb(*r, *g, *b));
        }
    }

    match first {
        "black" => Some(Rgba::BLACK),
        "white" => Some(Rgba::WHITE),
        "red" => Some(Rgba::rgb(220, 38, 38)),
        "green" => Some(Rgba::rgb(22, 163, 74)),
        "blue" => Some(Rgba::rgb(37, 99, 235)),
        "gray" | "grey" => Some(Rgba::rgb(107, 114, 128)),
        "silver" => Some(Rgba::rgb(192, 192, 192)),
        "navy" => Some(Rgba::rgb(0, 0, 128)),
        "teal" => Some(Rgba::rgb(0, 128, 128)),
        "purple" => Some(Rgba::rgb(128, 0, 128)),
        "orange" => Some(Rgba::rgb(234, 88, 12)),
        "yellow" => Some(Rgba::rgb(234, 179, 8)),
        _ => None,
    }
}

fn hex_digit(ch: char) -> Option<u8> {
    ch.to_digit(16).map(|value| value as u8)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{css, html, image_data::DecodedImage};

    use super::{PaintItem, layout_document, layout_document_with_images};

    #[test]
    fn produces_text_layout() {
        let dom = html::parse("<html><body><h1>Cherry Engine</h1><p>Hello Rust</p></body></html>");
        let sheet = css::parse_stylesheet("p { color: #123456; }");
        let doc = layout_document(&dom, &sheet, 800.0);
        assert!(doc.height > 0.0);
        assert!(
            doc.items
                .iter()
                .any(|item| matches!(item, PaintItem::Text(text) if text.text == "Cherry"))
        );
    }

    #[test]
    fn honors_display_none() {
        let dom = html::parse("<div>shown</div><p class='hide'>secret</p>");
        let sheet = css::parse_stylesheet(".hide { display: none; }");
        let doc = layout_document(&dom, &sheet, 600.0);
        assert!(
            !doc.items
                .iter()
                .any(|item| matches!(item, PaintItem::Text(text) if text.text == "secret"))
        );
    }

    #[test]
    fn emits_image_item_for_decoded_image() {
        let dom = html::parse("<img src='hero.png'>");
        let image_node = dom.find_first_tag("img").unwrap();
        let sheet = css::parse_stylesheet("");
        let mut images = HashMap::new();
        images.insert(
            image_node,
            DecodedImage {
                width: 320,
                height: 180,
                rgba: vec![0; 320 * 180 * 4],
            },
        );

        let doc = layout_document_with_images(&dom, &sheet, &images, 800.0);
        assert!(
            doc.items
                .iter()
                .any(|item| matches!(item, PaintItem::Image(image) if image.node == image_node))
        );
    }

    #[test]
    fn wraps_long_thai_text_without_ascii_spaces() {
        let dom = html::parse(
            "<p>ภาษาไทยทดสอบการตัดบรรทัดโดยไม่มีช่องว่างภาษาไทยทดสอบการตัดบรรทัดโดยไม่มีช่องว่าง</p>",
        );
        let sheet = css::parse_stylesheet("");
        let doc = layout_document(&dom, &sheet, 320.0);
        let ys = doc
            .items
            .iter()
            .filter_map(|item| match item {
                PaintItem::Text(text)
                    if text
                        .text
                        .chars()
                        .any(|ch| matches!(ch as u32, 0x0e00..=0x0e7f)) =>
                {
                    Some(text.rect.y)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(ys.len() > 3);
        assert!(ys.iter().any(|y| *y > ys[0] + 1.0));
        assert!(!doc.items.iter().any(
            |item| matches!(item, PaintItem::Text(text) if text.text == "่" || text.text == "้")
        ));
    }
}
