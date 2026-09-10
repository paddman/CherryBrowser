use std::collections::HashMap;

use crate::dom::{Dom, NodeId, NodeKind};

pub type Properties = HashMap<String, String>;

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
    pub source_order: usize,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Selector {
    parts: Vec<SimpleSelector>,
    specificity: u32,
}

#[derive(Debug, Clone, Default)]
struct SimpleSelector {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
}

pub fn parse_stylesheet(input: &str) -> Stylesheet {
    let cleaned = strip_comments(input);
    let mut rest = cleaned.as_str();
    let mut rules = Vec::new();
    let mut source_order = 0;

    while let Some(open) = rest.find('{') {
        let selector_text = rest[..open].trim();
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find('}') else {
            break;
        };
        let body = &after_open[..close];
        rest = &after_open[close + 1..];

        if selector_text.is_empty() || selector_text.starts_with('@') {
            continue;
        }

        let selectors = selector_text
            .split(',')
            .filter_map(parse_selector)
            .collect::<Vec<_>>();
        if selectors.is_empty() {
            continue;
        }

        let declarations = parse_declarations(body);
        if declarations.is_empty() {
            continue;
        }

        rules.push(Rule {
            selectors,
            declarations,
            source_order,
        });
        source_order += 1;
    }

    Stylesheet { rules }
}

pub fn parse_declarations(input: &str) -> Vec<Declaration> {
    input
        .split(';')
        .filter_map(|part| {
            let (name, value) = part.split_once(':')?;
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                return None;
            }

            let value = value
                .strip_suffix("!important")
                .unwrap_or(value)
                .trim()
                .to_string();

            Some(Declaration { name, value })
        })
        .collect()
}

pub fn cascade(
    dom: &Dom,
    node: NodeId,
    stylesheet: &Stylesheet,
    inherited: &Properties,
) -> Properties {
    let mut properties = Properties::new();

    for name in INHERITED_PROPERTIES {
        if let Some(value) = inherited.get(*name) {
            properties.insert((*name).to_string(), value.clone());
        }
    }

    let mut matching = Vec::new();
    for rule in &stylesheet.rules {
        let best_specificity = rule
            .selectors
            .iter()
            .filter(|selector| selector.matches(dom, node))
            .map(|selector| selector.specificity)
            .max();

        if let Some(specificity) = best_specificity {
            matching.push((specificity, rule.source_order, rule));
        }
    }

    matching.sort_by_key(|(specificity, source_order, _)| (*specificity, *source_order));

    for (_, _, rule) in matching {
        for declaration in &rule.declarations {
            properties.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    if let Some(inline) = dom.attr(node, "style") {
        for declaration in parse_declarations(inline) {
            properties.insert(declaration.name, declaration.value);
        }
    }

    properties
}

impl Selector {
    fn matches(&self, dom: &Dom, node: NodeId) -> bool {
        let Some((last, ancestors)) = self.parts.split_last() else {
            return false;
        };

        if !last.matches(dom, node) {
            return false;
        }

        let mut ancestor = dom.parent(node);
        for part in ancestors.iter().rev() {
            let mut found = None;
            while let Some(candidate) = ancestor {
                if part.matches(dom, candidate) {
                    found = Some(candidate);
                    break;
                }
                ancestor = dom.parent(candidate);
            }

            let Some(matched) = found else {
                return false;
            };
            ancestor = dom.parent(matched);
        }

        true
    }
}

impl SimpleSelector {
    fn matches(&self, dom: &Dom, node: NodeId) -> bool {
        let NodeKind::Element(element) = &dom.node(node).kind else {
            return false;
        };

        if let Some(tag) = &self.tag
            && element.tag_name != *tag
        {
            return false;
        }

        if let Some(id) = &self.id
            && element.attributes.get("id") != Some(id)
        {
            return false;
        }

        if !self.classes.is_empty() {
            let class_attr = element
                .attributes
                .get("class")
                .map(String::as_str)
                .unwrap_or("");
            let classes = class_attr.split_whitespace().collect::<Vec<_>>();
            if self
                .classes
                .iter()
                .any(|wanted| !classes.iter().any(|actual| *actual == wanted))
            {
                return false;
            }
        }

        true
    }
}

fn parse_selector(raw: &str) -> Option<Selector> {
    let normalized = raw
        .replace('>', " ")
        .replace('+', " ")
        .replace('~', " ");
    let parts = normalized
        .split_whitespace()
        .filter_map(parse_simple_selector)
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return None;
    }

    let mut specificity = 0;
    for part in &parts {
        specificity += if part.id.is_some() { 100 } else { 0 };
        specificity += part.classes.len() as u32 * 10;
        specificity += if part.tag.is_some() { 1 } else { 0 };
    }

    Some(Selector { parts, specificity })
}

fn parse_simple_selector(raw: &str) -> Option<SimpleSelector> {
    let raw = raw.trim();
    if raw.is_empty() || raw == "*" {
        return Some(SimpleSelector::default());
    }

    let chars = raw.chars().collect::<Vec<_>>();
    let mut pos = 0;
    let mut selector = SimpleSelector::default();

    if chars
        .get(pos)
        .is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == '_')
    {
        let start = pos;
        while chars.get(pos).is_some_and(|ch| {
            ch.is_ascii_alphanumeric() || matches!(*ch, '-' | '_')
        }) {
            pos += 1;
        }
        selector.tag = Some(
            chars[start..pos]
                .iter()
                .collect::<String>()
                .to_ascii_lowercase(),
        );
    }

    while pos < chars.len() {
        match chars[pos] {
            '#' | '.' => {
                let marker = chars[pos];
                pos += 1;
                let start = pos;
                while chars
                    .get(pos)
                    .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(*ch, '-' | '_'))
                {
                    pos += 1;
                }
                if start == pos {
                    continue;
                }
                let value = chars[start..pos].iter().collect::<String>();
                if marker == '#' {
                    selector.id = Some(value);
                } else {
                    selector.classes.push(value);
                }
            }
            ':' | '[' => {
                // Pseudo classes/elements and attribute selectors are intentionally
                // ignored in milestone 0.1 rather than pretending to implement them.
                break;
            }
            _ => pos += 1,
        }
    }

    Some(selector)
}

fn strip_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    loop {
        let Some(start) = rest.find("/*") else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..start]);

        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("*/") else {
            break;
        };
        rest = &after_start[end + 2..];
    }

    out
}

const INHERITED_PROPERTIES: &[&str] = &[
    "color",
    "font-family",
    "font-size",
    "font-style",
    "font-weight",
    "line-height",
    "text-decoration",
];

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::html;

    use super::{cascade, parse_stylesheet};

    #[test]
    fn parses_and_matches_descendant_selectors() {
        let dom = html::parse(r#"<div class="card"><p id="lead">Cherry</p></div>"#);
        let sheet = parse_stylesheet(
            r#"
            p { color: black; }
            .card #lead { color: #00aaff; font-size: 20px; }
            "#,
        );
        let p = dom.find_first_tag("p").unwrap();
        let style = cascade(&dom, p, &sheet, &HashMap::new());
        assert_eq!(style.get("color").map(String::as_str), Some("#00aaff"));
        assert_eq!(style.get("font-size").map(String::as_str), Some("20px"));
    }

    #[test]
    fn inline_style_wins() {
        let dom = html::parse(r#"<p style="color: red">x</p>"#);
        let sheet = parse_stylesheet("p { color: blue; }");
        let p = dom.find_first_tag("p").unwrap();
        let style = cascade(&dom, p, &sheet, &HashMap::new());
        assert_eq!(style.get("color").map(String::as_str), Some("red"));
    }
}
