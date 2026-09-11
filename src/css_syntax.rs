#[derive(Debug, Default)]
struct ScanState {
    quote: Option<u8>,
    paren_depth: usize,
    bracket_depth: usize,
    brace_depth: usize,
}

impl ScanState {
    fn is_top_level(&self) -> bool {
        self.quote.is_none()
            && self.paren_depth == 0
            && self.bracket_depth == 0
            && self.brace_depth == 0
    }

    fn consume_byte(&mut self, bytes: &[u8], pos: &mut usize) {
        let byte = bytes[*pos];

        if let Some(active_quote) = self.quote {
            if byte == b'\\' {
                *pos = (*pos + 2).min(bytes.len());
                return;
            }
            if byte == active_quote {
                self.quote = None;
            }
            *pos += 1;
            return;
        }

        if byte == b'\\' {
            *pos = (*pos + 2).min(bytes.len());
            return;
        }

        match byte {
            b'\'' | b'"' => self.quote = Some(byte),
            b'(' => self.paren_depth += 1,
            b')' => self.paren_depth = self.paren_depth.saturating_sub(1),
            b'[' => self.bracket_depth += 1,
            b']' => self.bracket_depth = self.bracket_depth.saturating_sub(1),
            b'{' => self.brace_depth += 1,
            b'}' => self.brace_depth = self.brace_depth.saturating_sub(1),
            _ => {}
        }
        *pos += 1;
    }
}

pub(crate) fn strip_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut quote = None;
    let mut segment_start = 0;
    let mut pos = 0;

    while pos < bytes.len() {
        let byte = bytes[pos];
        if let Some(active_quote) = quote {
            if byte == b'\\' {
                pos = (pos + 2).min(bytes.len());
                continue;
            }
            if byte == active_quote {
                quote = None;
            }
            pos += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => {
                quote = Some(byte);
                pos += 1;
            }
            b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                out.push_str(&input[segment_start..pos]);
                let comment_start = pos;
                pos += 2;
                while pos + 1 < bytes.len()
                    && !(bytes[pos] == b'*' && bytes[pos + 1] == b'/')
                {
                    pos += 1;
                }

                if pos + 1 >= bytes.len() {
                    if comment_start > segment_start {
                        out.push(' ');
                    }
                    return out;
                }

                pos += 2;
                out.push(' ');
                segment_start = pos;
            }
            _ => pos += 1,
        }
    }

    out.push_str(&input[segment_start..]);
    out
}

pub(crate) fn split_top_level(input: &str, delimiter: u8) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut state = ScanState::default();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut pos = 0;

    while pos < bytes.len() {
        if state.is_top_level() && bytes[pos] == delimiter {
            parts.push(&input[start..pos]);
            pos += 1;
            start = pos;
            continue;
        }
        state.consume_byte(bytes, &mut pos);
    }

    parts.push(&input[start..]);
    parts
}

pub(crate) fn split_once_top_level(input: &str, delimiter: u8) -> Option<(&str, &str)> {
    let pos = find_top_level_char(input, 0, delimiter)?;
    Some((&input[..pos], &input[pos + 1..]))
}

pub(crate) fn find_top_level_char(input: &str, start: usize, target: u8) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut state = ScanState::default();
    let mut pos = start.min(bytes.len());

    while pos < bytes.len() {
        if state.is_top_level() && bytes[pos] == target {
            return Some(pos);
        }
        state.consume_byte(bytes, &mut pos);
    }

    None
}

pub(crate) fn strip_trailing_important(value: &str) -> (&str, bool) {
    let bytes = value.as_bytes();
    let mut state = ScanState::default();
    let mut last_bang = None;
    let mut pos = 0;

    while pos < bytes.len() {
        if state.is_top_level() && bytes[pos] == b'!' {
            last_bang = Some(pos);
            pos += 1;
            continue;
        }
        state.consume_byte(bytes, &mut pos);
    }

    let Some(bang) = last_bang else {
        return (value.trim(), false);
    };
    let suffix = value[bang + 1..].trim();
    if suffix.eq_ignore_ascii_case("important") {
        (value[..bang].trim(), true)
    } else {
        (value.trim(), false)
    }
}

pub(crate) fn skip_at_rule(input: &str, start: usize) -> usize {
    let bytes = input.as_bytes();
    let mut state = ScanState::default();
    let mut pos = start.min(bytes.len());

    while pos < bytes.len() {
        if state.is_top_level() {
            match bytes[pos] {
                b';' => return pos + 1,
                b'{' => {
                    return find_matching_brace(input, pos)
                        .map_or(input.len(), |close| close + 1);
                }
                _ => {}
            }
        }
        state.consume_byte(bytes, &mut pos);
    }

    input.len()
}

pub(crate) fn find_matching_brace(input: &str, open: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.get(open) != Some(&b'{') {
        return None;
    }

    let mut quote = None;
    let mut depth = 0usize;
    let mut pos = open;

    while pos < bytes.len() {
        let byte = bytes[pos];
        if let Some(active_quote) = quote {
            if byte == b'\\' {
                pos = (pos + 2).min(bytes.len());
                continue;
            }
            if byte == active_quote {
                quote = None;
            }
            pos += 1;
            continue;
        }

        if byte == b'\\' {
            pos = (pos + 2).min(bytes.len());
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
        pos += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        find_top_level_char, split_once_top_level, split_top_level, strip_comments,
        strip_trailing_important,
    };

    #[test]
    fn comments_inside_strings_are_preserved() {
        let cleaned = strip_comments(r#"p { content: "/* keep */"; color: red /* drop */; }"#);
        assert!(cleaned.contains("/* keep */"));
        assert!(!cleaned.contains("drop"));
    }

    #[test]
    fn splits_only_top_level_semicolons() {
        let parts = split_top_level(
            r#"background:url("data:image/svg+xml;a:b");content:"x;y";color:red"#,
            b';',
        );
        assert_eq!(parts.len(), 3);
        assert!(parts[0].contains("data:image/svg+xml;a:b"));
        assert_eq!(parts[1], r#"content:"x;y""#);
    }

    #[test]
    fn finds_only_top_level_property_colon() {
        let (name, value) = split_once_top_level(
            r#"background:url("data:image/svg+xml;a:b")"#,
            b':',
        )
        .unwrap();
        assert_eq!(name, "background");
        assert!(value.contains("a:b"));
    }

    #[test]
    fn important_inside_string_is_not_a_priority_marker() {
        assert_eq!(
            strip_trailing_important(r#""literal !important""#),
            (r#""literal !important""#, false)
        );
        assert_eq!(strip_trailing_important("red ! IMPORTANT"), ("red", true));
    }

    #[test]
    fn rule_open_brace_ignores_nested_and_quoted_braces() {
        let selector = r#"a[data-x="{"]:not(.x) { color:red }"#;
        let open = find_top_level_char(selector, 0, b'{').unwrap();
        assert_eq!(&selector[open..open + 1], "{");
        assert!(open > selector.find("\"{\"").unwrap());
    }
}
