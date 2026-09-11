pub fn decode_web_text(bytes: &[u8], content_type: &str, sniff_html: bool) -> String {
    if let Some((encoding, skip)) = bom_encoding(bytes) {
        return decode_as(&bytes[skip..], encoding);
    }

    let label = charset_label(content_type)
        .or_else(|| sniff_html.then(|| sniff_html_charset(bytes)).flatten());

    if let Some(label) = label
        && let Some(encoding) = encoding_from_label(&label)
    {
        return decode_as(bytes, encoding);
    }

    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => decode_windows_1252(bytes),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
    Windows1252,
    Windows874,
}

fn bom_encoding(bytes: &[u8]) -> Option<(WebEncoding, usize)> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        Some((WebEncoding::Utf8, 3))
    } else if bytes.starts_with(&[0xff, 0xfe]) {
        Some((WebEncoding::Utf16Le, 2))
    } else if bytes.starts_with(&[0xfe, 0xff]) {
        Some((WebEncoding::Utf16Be, 2))
    } else {
        None
    }
}

fn charset_label(input: &str) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    let start = lower.find("charset")? + "charset".len();
    let rest = lower[start..].trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.trim_start_matches(['\'', '"']);
    let label = rest
        .split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, ';' | '\'' | '"' | '>'))
        .next()?
        .trim();
    (!label.is_empty()).then(|| label.to_string())
}

fn sniff_html_charset(bytes: &[u8]) -> Option<String> {
    let preview_len = bytes.len().min(1024);
    let preview = String::from_utf8_lossy(&bytes[..preview_len]).to_ascii_lowercase();
    let mut cursor = preview.as_str();

    while let Some(meta_pos) = cursor.find("<meta") {
        let after_meta = &cursor[meta_pos + 5..];
        let tag_end = after_meta.find('>').unwrap_or(after_meta.len());
        let tag = &after_meta[..tag_end];
        if let Some(label) = charset_label(tag) {
            return Some(label);
        }

        if tag_end >= after_meta.len() {
            break;
        }
        cursor = &after_meta[tag_end + 1..];
    }

    None
}

fn encoding_from_label(label: &str) -> Option<WebEncoding> {
    match label.trim().to_ascii_lowercase().as_str() {
        "utf-8" | "utf8" | "unicode-1-1-utf-8" => Some(WebEncoding::Utf8),
        "utf-16" | "utf-16le" => Some(WebEncoding::Utf16Le),
        "utf-16be" => Some(WebEncoding::Utf16Be),
        "windows-1252" | "cp1252" | "x-cp1252" | "iso-8859-1" | "latin1"
        | "latin-1" | "ascii" | "us-ascii" => Some(WebEncoding::Windows1252),
        "windows-874" | "dos-874" | "iso-8859-11" | "tis-620" => {
            Some(WebEncoding::Windows874)
        }
        _ => None,
    }
}

fn decode_as(bytes: &[u8], encoding: WebEncoding) -> String {
    match encoding {
        WebEncoding::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        WebEncoding::Utf16Le => decode_utf16(bytes, true),
        WebEncoding::Utf16Be => decode_utf16(bytes, false),
        WebEncoding::Windows1252 => decode_windows_1252(bytes),
        WebEncoding::Windows874 => decode_windows_874(bytes),
    }
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
    let units = bytes.chunks_exact(2).map(|pair| {
        let pair = [pair[0], pair[1]];
        if little_endian {
            u16::from_le_bytes(pair)
        } else {
            u16::from_be_bytes(pair)
        }
    });
    char::decode_utf16(units)
        .map(|result| result.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

fn decode_windows_1252(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| windows_1252_char(*byte)).collect()
}

fn windows_1252_char(byte: u8) -> char {
    match byte {
        0x80 => '\u{20ac}',
        0x82 => '\u{201a}',
        0x83 => '\u{0192}',
        0x84 => '\u{201e}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{02c6}',
        0x89 => '\u{2030}',
        0x8a => '\u{0160}',
        0x8b => '\u{2039}',
        0x8c => '\u{0152}',
        0x8e => '\u{017d}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201c}',
        0x94 => '\u{201d}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{02dc}',
        0x99 => '\u{2122}',
        0x9a => '\u{0161}',
        0x9b => '\u{203a}',
        0x9c => '\u{0153}',
        0x9e => '\u{017e}',
        0x9f => '\u{0178}',
        _ => char::from_u32(u32::from(byte)).unwrap_or(char::REPLACEMENT_CHARACTER),
    }
}

fn decode_windows_874(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| windows_874_char(*byte)).collect()
}

fn windows_874_char(byte: u8) -> char {
    match byte {
        0x80 => '\u{20ac}',
        0x85 => '\u{2026}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201c}',
        0x94 => '\u{201d}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0xa0 => '\u{00a0}',
        0xa1..=0xda => char::from_u32(0x0e01 + u32::from(byte - 0xa1))
            .unwrap_or(char::REPLACEMENT_CHARACTER),
        0xdf => '\u{0e3f}',
        0xe0..=0xfb => char::from_u32(0x0e40 + u32::from(byte - 0xe0))
            .unwrap_or(char::REPLACEMENT_CHARACTER),
        0xdb..=0xde | 0xfc..=0xff => char::REPLACEMENT_CHARACTER,
        _ => char::from_u32(u32::from(byte)).unwrap_or(char::REPLACEMENT_CHARACTER),
    }
}

#[cfg(test)]
mod tests {
    use super::decode_web_text;

    #[test]
    fn utf8_bom_wins() {
        assert_eq!(
            decode_web_text(b"\xef\xbb\xbfCherry", "text/html; charset=windows-1252", true),
            "Cherry"
        );
    }

    #[test]
    fn decodes_windows_1252_from_http_header() {
        assert_eq!(
            decode_web_text(b"caf\xe9 \x80", "text/plain; charset=windows-1252", false),
            "caf\u{e9} \u{20ac}"
        );
    }

    #[test]
    fn sniffs_windows_874_from_meta_charset() {
        let mut bytes = b"<meta charset=windows-874><p>".to_vec();
        bytes.extend_from_slice(&[0xa1, 0xd2, 0xc3]);
        assert!(decode_web_text(&bytes, "text/html", true).contains("การ"));
    }

    #[test]
    fn decodes_utf16le_bom() {
        let bytes = [0xff, 0xfe, b'C', 0, b'B', 0];
        assert_eq!(decode_web_text(&bytes, "text/html", true), "CB");
    }

    #[test]
    fn invalid_utf8_without_charset_falls_back_to_windows_1252() {
        assert_eq!(decode_web_text(b"caf\xe9", "text/plain", false), "caf\u{e9}");
    }
}
