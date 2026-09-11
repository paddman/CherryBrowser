use std::collections::HashMap;

use crate::{
    css_syntax,
    dom::{Dom, NodeId, NodeKind},
};

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
    pub important: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CascadePriority {
    important: u8,
    inline: u8,
    specificity: u32,
    source_order: usize,
    declaration_order: usize,
}

pub fn parse_stylesheet(input: &str) -> Stylesheet {
    let cleaned = css_syntax::strip_comments(input);
    let mut rules = Vec::new();
    let mut source_order = 0;
    let mut pos = 0;

    while pos < cleaned.len() {
        pos = skip_ascii_whitespace(&cleaned, pos);
        if pos >= cleaned.len() {
            break;
        }

        if cleaned.as_bytes()[pos] == b'@' {
            pos = css_syntax::skip_at_rule(&cleaned, pos);
            continue;
        }

        let Some(open) = css_syntax::find_top_level_char(&cleaned, pos, b'{') else {
            break;
        };
        let selector_text = cleaned[pos..open].trim();
        let Some(close) = css_syntax::find_matching_brace(&cleaned, open) else {
            break;
        };
        let body = &cleaned[open + 1..close];
        pos = close + 1;

        if selector_text.is_empty() {
            continue;
        }

        let selector_parts = css_syntax::split_top_level(selector_text, b',');
        let Some(selectors) = selector_parts
            .into_iter()
            .map(|selector| parse_selector(selector.trim()))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
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
    let cleaned = css_syntax::strip_comments(input);
    css_syntax::split_top_level(&cleaned, b';')
        .into_iter()
        .filter_map(|part| {
            let (name, value) = css_syntax::split_once_top_level(part, b':')?;
            let name = normalize_property_name(name.trim())?;
            let (value, important) = css_syntax::strip_trailing_important(value);
            if value.is_empty() {
                return None;
            }

            Some(Declaration {
                name,
                value: value.to_string(),
                important,
            })
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
    let mut winners: HashMap<String, (CascadePriority, String)> = HashMap::new();

    for name in INHERITED_PROPERTIES {
        if let Some(value) = inherited.get(*name) {
            properties.insert((*name).to_string(), value.clone());
        }
    }
    for (name, value) in inherited {
        if name.starts_with("--") {
            properties.insert(name.clone(), value.clone());
        }
    }

    for rule in &stylesheet.rules {
        let best_specificity = rule
            .selectors
            .iter()
            .filter(|selector| selector.matches(dom, node))
            .map(|selector| selector.specificity)
            .max();

        let Some(specificity) = best_specificity else {
            continue;
        };

        for (declaration_order, declaration) in rule.declarations.iter().enumerate() {
            let priority = CascadePriority {
                important: u8::from(declaration.important),
                inline: 0,
                specificity,
                source_order: rule.source_order,
                declaration_order,
            };
            apply_candidate(&mut winners, declaration, priority);
        }
    }

    if let Some(inline) = dom.attr(node, "style") {
        for (declaration_order, declaration) in parse_declarations(inline).iter().enumerate() {
            let priority = CascadePriority {
                important: u8::from(declaration.important),
                inline: 1,
                specificity: u32::MAX,
                source_order: usize::MAX,
                declaration_order,
            };
            apply_candidate(&mut winners, declaration, priority);
        }
    }

    for (name, (_, value)) in winners {
        properties.insert(name, value);
    }

    properties
}

fn apply_candidate(
    winners: &mut HashMap<String, (CascadePriority, String)>,
    declaration: &Declaration,
    priority: CascadePriority,
) {
    let should_replace = winners
        .get(&declaration.name)
        .is_none_or(|(current, _)| priority >= *current);

    if should_replace {
        winners.insert(
            declaration.name.clone(),
            (priority, declaration.value.clone()),
        );
    }
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
    if raw.contains('>') || raw.contains('+') || raw.contains('~') {
        return None;
    }

    let parts = raw
        .split_whitespace()
        .map(parse_simple_selector)
        .collect::<Option<Vec<_>>>()?;

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

    if chars.get(pos) == Some(&'*') {
        pos += 1;
    } else if chars
        .get(pos)
        .is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == '_')
    {
        let start = pos;
        while chars
            .get(pos)
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(*ch, '-' | '_'))
        {
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
                    return None;
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
                // unsupported for now. Reject this selector instead of silently
                // broadening the match and applying styles to the wrong elements.
                return None;
            }
            _ => return None,
        }
    }

    Some(selector)
}

fn normalize_property_name(name: &str) -> Option<String> {
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return None;
    }

    if name.starts_with("--") {
        Some(name.to_string())
    } else {
        Some(name.to_ascii_lowercase())
    }
}

fn skip_ascii_whitespace(input: &str, mut pos: usize) -> usize {
    while input
        .as_bytes()
        .get(pos)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        pos += 1;
    }
    pos
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

    use super::{Properties, cascade, parse_declarations, parse_stylesheet};

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
    fn inline_style_wins_normal_author_rule() {
        let dom = html::parse(r#"<p style="color: red">x</p>"#);
        let sheet = parse_stylesheet("p { color: blue; }");
        let p = dom.find_first_tag("p").unwrap();
        let style = cascade(&dom, p, &sheet, &HashMap::new());
        assert_eq!(style.get("color").map(String::as_str), Some("red"));
    }

    #[test]
    fn important_author_rule_beats_normal_inline_style() {
        let dom = html::parse(r#"<p id="lead" style="color: green">x</p>"#);
        let sheet = parse_stylesheet("#lead { color: blue !important; } p { color: red; }");
        let p = dom.find_first_tag("p").unwrap();
        let style = cascade(&dom, p, &sheet, &HashMap::new());
        assert_eq!(style.get("color").map(String::as_str), Some("blue"));
    }

    #[test]
    fn important_inline_style_beats_important_author_rule() {
        let dom = html::parse(r#"<p id="lead" style="color: green !IMPORTANT">x</p>"#);
        let sheet = parse_stylesheet("#lead { color: blue !important; }");
        let p = dom.find_first_tag("p").unwrap();
        let style = cascade(&dom, p, &sheet, &HashMap::new());
        assert_eq!(style.get("color").map(String::as_str), Some("green"));
    }

    #[test]
    fn skips_unsupported_at_rule_blocks_without_eating_following_rules() {
        let sheet = parse_stylesheet(
            r#"
            @charset "utf-8";
            @media print { p { color: black; } }
            p { color: blue; }
            "#,
        );
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn rejects_unsupported_combinators_instead_of_broadening_them() {
        let sheet = parse_stylesheet("div > p { color: red; } p { color: blue; }");
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn rejects_entire_selector_when_one_descendant_part_is_unsupported() {
        let sheet = parse_stylesheet(".card :hover { color: red; } .card { color: blue; }");
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn rejects_entire_selector_list_when_one_selector_is_unsupported() {
        let sheet = parse_stylesheet("p, div > span { color: red; } p { color: blue; }");
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn declaration_parser_preserves_nested_colons_and_semicolons() {
        let declarations = parse_declarations(
            r#"background-image:url("data:image/svg+xml;utf8,<svg></svg>");content:"a:b;c";color:red ! IMPORTANT"#,
        );
        assert_eq!(declarations.len(), 3);
        assert!(declarations[0].value.contains("data:image/svg+xml;utf8"));
        assert_eq!(declarations[1].value, r#""a:b;c""#);
        assert_eq!(declarations[2].value, "red");
        assert!(declarations[2].important);
    }

    #[test]
    fn comments_inside_css_strings_are_not_removed() {
        let declarations = parse_declarations(r#"content:"/* keep */";color:red/* drop */"#);
        assert_eq!(declarations.len(), 2);
        assert_eq!(declarations[0].value, r#""/* keep */""#);
        assert_eq!(declarations[1].value, "red");
    }

    #[test]
    fn custom_property_names_preserve_case_and_inherit() {
        let dom = html::parse(r#"<p style="--BrandColor: blue">x</p>"#);
        let p = dom.find_first_tag("p").unwrap();
        let mut inherited = Properties::new();
        inherited.insert("--ThemeColor".to_string(), "red".to_string());

        let style = cascade(&dom, p, &parse_stylesheet(""), &inherited);
        assert_eq!(style.get("--ThemeColor").map(String::as_str), Some("red"));
        assert_eq!(style.get("--BrandColor").map(String::as_str), Some("blue"));
        assert!(!style.contains_key("--brandcolor"));
    }
}
