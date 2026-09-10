use std::collections::HashMap;

use crate::dom::{Dom, NodeId};

pub fn parse(input: &str) -> Dom {
    HtmlParser::new(input).parse()
}

struct HtmlParser {
    chars: Vec<char>,
    pos: usize,
    dom: Dom,
    stack: Vec<NodeId>,
}

impl HtmlParser {
    fn new(input: &str) -> Self {
        let dom = Dom::new();
        let root = dom.root();
        Self {
            chars: input.chars().collect(),
            pos: 0,
            dom,
            stack: vec![root],
        }
    }

    fn parse(mut self) -> Dom {
        while !self.eof() {
            if self.peek() == Some('<') {
                if self.starts_with("<!--") {
                    self.consume_comment();
                } else if self.starts_with("</") {
                    self.consume_end_tag();
                } else if self.starts_with("<!") || self.starts_with("<?") {
                    self.consume_declaration();
                } else {
                    self.consume_start_tag();
                }
            } else {
                self.consume_text();
            }
        }
        self.dom
    }

    fn current_parent(&self) -> NodeId {
        *self.stack.last().expect("parser stack always has document")
    }

    fn consume_comment(&mut self) {
        self.pos += 4;
        while !self.eof() && !self.starts_with("-->") {
            self.pos += 1;
        }
        if self.starts_with("-->") {
            self.pos += 3;
        }
    }

    fn consume_declaration(&mut self) {
        while !self.eof() && self.peek() != Some('>') {
            self.pos += 1;
        }
        if self.peek() == Some('>') {
            self.pos += 1;
        }
    }

    fn consume_end_tag(&mut self) {
        self.pos += 2;
        self.skip_whitespace();
        let tag = self.consume_name();
        while !self.eof() && self.peek() != Some('>') {
            self.pos += 1;
        }
        if self.peek() == Some('>') {
            self.pos += 1;
        }

        if tag.is_empty() {
            return;
        }

        while self.stack.len() > 1 {
            let open = self.stack.pop().expect("stack length checked");
            if self.dom.tag_name(open) == Some(tag.as_str()) {
                break;
            }
        }
    }

    fn consume_start_tag(&mut self) {
        self.pos += 1;
        self.skip_whitespace();
        let tag = self.consume_name();

        if tag.is_empty() {
            self.consume_until_gt();
            return;
        }

        let mut attributes = HashMap::new();
        let mut self_closing = false;

        loop {
            self.skip_whitespace();
            if self.eof() {
                break;
            }
            if self.starts_with("/>") {
                self.pos += 2;
                self_closing = true;
                break;
            }
            if self.peek() == Some('>') {
                self.pos += 1;
                break;
            }

            let name = self.consume_name();
            if name.is_empty() {
                self.pos += 1;
                continue;
            }

            self.skip_whitespace();
            let value = if self.peek() == Some('=') {
                self.pos += 1;
                self.skip_whitespace();
                self.consume_attribute_value()
            } else {
                String::new()
            };
            attributes.insert(name, decode_entities(&value));
        }

        let parent = self.current_parent();
        let node = self.dom.append_element(parent, tag.clone(), attributes);

        if !self_closing && !is_void_element(&tag) {
            self.stack.push(node);

            if matches!(tag.as_str(), "script" | "style") {
                self.consume_raw_text(&tag, node);
            }
        }
    }

    fn consume_raw_text(&mut self, tag: &str, parent: NodeId) {
        let needle = format!("</{tag}");
        let start = self.pos;
        while !self.eof() && !self.starts_with_case_insensitive(&needle) {
            self.pos += 1;
        }

        if self.pos > start {
            let text: String = self.chars[start..self.pos].iter().collect();
            self.dom.append_text(parent, text);
        }
    }

    fn consume_text(&mut self) {
        let start = self.pos;
        while !self.eof() && self.peek() != Some('<') {
            self.pos += 1;
        }
        if self.pos > start {
            let raw: String = self.chars[start..self.pos].iter().collect();
            let text = decode_entities(&raw);
            if !text.is_empty() {
                let parent = self.current_parent();
                self.dom.append_text(parent, text);
            }
        }
    }

    fn consume_name(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.') {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase()
    }

    fn consume_attribute_value(&mut self) -> String {
        match self.peek() {
            Some(quote @ ('"' | '\'')) => {
                self.pos += 1;
                let start = self.pos;
                while !self.eof() && self.peek() != Some(quote) {
                    self.pos += 1;
                }
                let value: String = self.chars[start..self.pos].iter().collect();
                if self.peek() == Some(quote) {
                    self.pos += 1;
                }
                value
            }
            _ => {
                let start = self.pos;
                while let Some(ch) = self.peek() {
                    if ch.is_whitespace() || matches!(ch, '>' | '/') {
                        break;
                    }
                    self.pos += 1;
                }
                self.chars[start..self.pos].iter().collect()
            }
        }
    }

    fn consume_until_gt(&mut self) {
        while !self.eof() && self.peek() != Some('>') {
            self.pos += 1;
        }
        if self.peek() == Some('>') {
            self.pos += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    fn starts_with(&self, needle: &str) -> bool {
        let needle: Vec<char> = needle.chars().collect();
        self.chars
            .get(self.pos..self.pos + needle.len())
            .is_some_and(|slice| slice == needle.as_slice())
    }

    fn starts_with_case_insensitive(&self, needle: &str) -> bool {
        let needle: Vec<char> = needle.chars().collect();
        self.chars
            .get(self.pos..self.pos + needle.len())
            .is_some_and(|slice| {
                slice
                    .iter()
                    .zip(needle.iter())
                    .all(|(a, b)| a.eq_ignore_ascii_case(b))
            })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eof(&self) -> bool {
        self.pos >= self.chars.len()
    }
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn decode_entities(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < chars.len() {
        if chars[pos] != '&' {
            out.push(chars[pos]);
            pos += 1;
            continue;
        }

        let Some(relative_end) = chars[pos + 1..].iter().position(|ch| *ch == ';') else {
            out.push('&');
            pos += 1;
            continue;
        };

        let end = pos + 1 + relative_end;
        if end.saturating_sub(pos) > 16 {
            out.push('&');
            pos += 1;
            continue;
        }

        let entity: String = chars[pos + 1..end].iter().collect();
        let decoded = match entity.as_str() {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" => Some(' '),
            _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                u32::from_str_radix(&entity[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            _ if entity.starts_with('#') => {
                entity[1..].parse::<u32>().ok().and_then(char::from_u32)
            }
            _ => None,
        };

        if let Some(ch) = decoded {
            out.push(ch);
            pos = end + 1;
        } else {
            out.push('&');
            pos += 1;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use crate::dom::NodeKind;

    use super::parse;

    #[test]
    fn parses_elements_attributes_and_text() {
        let dom = parse(r#"<main id="x"><h1>Hello &amp; Cherry</h1><br><p>Rust</p></main>"#);
        let main = dom.find_first_tag("main").unwrap();
        assert_eq!(dom.attr(main, "id"), Some("x"));
        assert!(dom.text_content(main).contains("Hello & Cherry"));
        assert!(
            dom.nodes()
                .iter()
                .any(|n| matches!(&n.kind, NodeKind::Element(e) if e.tag_name == "br"))
        );
    }

    #[test]
    fn keeps_script_markup_as_raw_text() {
        let dom = parse("<script>if (a < b) { x = '<div>'; }</script><p>ok</p>");
        let script = dom.find_first_tag("script").unwrap();
        assert!(dom.text_content(script).contains("a < b"));
        assert_eq!(dom.text_content(dom.find_first_tag("p").unwrap()), "ok");
    }
}
