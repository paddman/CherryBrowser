use std::sync::{Arc, OnceLock, RwLock};

type TextMeasurer = dyn Fn(&str, f32, bool) -> f32 + Send + Sync + 'static;

static TEXT_MEASURER: OnceLock<RwLock<Option<Arc<TextMeasurer>>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextUnit {
    pub text: String,
    pub is_space: bool,
}

pub(crate) fn set_text_measurer(measurer: Arc<TextMeasurer>) {
    let state = TEXT_MEASURER.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = state.write() {
        *guard = Some(measurer);
    }
}

pub(crate) fn line_break_units(text: &str) -> Vec<TextUnit> {
    let clusters = text_clusters(text);
    let mut units = Vec::new();
    let mut buffered = String::new();
    let mut pending_space = false;

    for cluster in clusters {
        if cluster.chars().all(char::is_whitespace) {
            flush_buffer(&mut units, &mut buffered);
            pending_space = true;
            continue;
        }

        if pending_space {
            units.push(TextUnit {
                text: " ".to_string(),
                is_space: true,
            });
            pending_space = false;
        }

        if is_breakable_cluster(&cluster) {
            flush_buffer(&mut units, &mut buffered);
            units.push(TextUnit {
                text: cluster,
                is_space: false,
            });
        } else {
            let break_after = cluster
                .chars()
                .last()
                .is_some_and(is_soft_break_punctuation);
            buffered.push_str(&cluster);
            if break_after {
                flush_buffer(&mut units, &mut buffered);
            }
        }
    }

    flush_buffer(&mut units, &mut buffered);
    units
}

pub(crate) fn estimated_text_width(text: &str, font_size: f32, monospace: bool) -> f32 {
    if let Some(width) = measured_text_width(text, font_size, monospace) {
        return width;
    }

    fallback_text_width(text, font_size, monospace)
}

fn measured_text_width(text: &str, font_size: f32, monospace: bool) -> Option<f32> {
    let state = TEXT_MEASURER.get()?;
    let guard = state.read().ok()?;
    let measurer = guard.as_ref()?;
    let width = measurer(text, font_size, monospace);
    (width.is_finite() && width >= 0.0).then_some(width)
}

fn fallback_text_width(text: &str, font_size: f32, monospace: bool) -> f32 {
    text_clusters(text)
        .iter()
        .map(|cluster| estimated_cluster_em(cluster, monospace))
        .sum::<f32>()
        * font_size
}

fn flush_buffer(units: &mut Vec<TextUnit>, buffered: &mut String) {
    if buffered.is_empty() {
        return;
    }
    units.push(TextUnit {
        text: std::mem::take(buffered),
        is_space: false,
    });
}

fn text_clusters(text: &str) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut clusters = Vec::new();
    let mut pos = 0;

    while pos < chars.len() {
        let mut cluster = String::new();
        let first = chars[pos];
        cluster.push(first);
        pos += 1;

        if first.is_whitespace() {
            while chars.get(pos).is_some_and(|ch| ch.is_whitespace()) {
                cluster.push(chars[pos]);
                pos += 1;
            }
            clusters.push(cluster);
            continue;
        }

        if is_regional_indicator(first)
            && chars.get(pos).is_some_and(|ch| is_regional_indicator(*ch))
        {
            cluster.push(chars[pos]);
            pos += 1;
        }

        if is_thai_leading_vowel(first)
            && chars
                .get(pos)
                .is_some_and(|ch| is_thai_base(*ch) || ch.is_alphabetic())
        {
            cluster.push(chars[pos]);
            pos += 1;
        }

        consume_cluster_extensions(&chars, &mut pos, &mut cluster);

        if cluster.chars().any(is_thai_char) {
            while chars.get(pos).is_some_and(|ch| is_thai_postposed(*ch)) {
                cluster.push(chars[pos]);
                pos += 1;
                consume_cluster_extensions(&chars, &mut pos, &mut cluster);
            }
        }

        clusters.push(cluster);
    }

    clusters
}

fn consume_cluster_extensions(chars: &[char], pos: &mut usize, cluster: &mut String) {
    loop {
        while chars.get(*pos).is_some_and(|ch| is_extension(*ch)) {
            cluster.push(chars[*pos]);
            *pos += 1;
        }

        if chars.get(*pos) == Some(&'\u{200d}') && *pos + 1 < chars.len() {
            cluster.push('\u{200d}');
            *pos += 1;
            cluster.push(chars[*pos]);
            *pos += 1;
            continue;
        }
        break;
    }
}

fn is_extension(ch: char) -> bool {
    is_combining_mark(ch) || is_variation_selector(ch) || is_emoji_modifier(ch)
}

fn is_combining_mark(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0300..=0x036f
            | 0x1ab0..=0x1aff
            | 0x1dc0..=0x1dff
            | 0x20d0..=0x20ff
            | 0xfe20..=0xfe2f
            | 0x0e31
            | 0x0e34..=0x0e3a
            | 0x0e47..=0x0e4e
    )
}

fn is_variation_selector(ch: char) -> bool {
    matches!(ch as u32, 0xfe00..=0xfe0f | 0xe0100..=0xe01ef)
}

fn is_emoji_modifier(ch: char) -> bool {
    matches!(ch as u32, 0x1f3fb..=0x1f3ff)
}

fn is_regional_indicator(ch: char) -> bool {
    matches!(ch as u32, 0x1f1e6..=0x1f1ff)
}

fn is_thai_char(ch: char) -> bool {
    matches!(ch as u32, 0x0e00..=0x0e7f)
}

fn is_thai_base(ch: char) -> bool {
    matches!(ch as u32, 0x0e01..=0x0e2e)
}

fn is_thai_leading_vowel(ch: char) -> bool {
    matches!(ch as u32, 0x0e40..=0x0e44)
}

fn is_thai_postposed(ch: char) -> bool {
    matches!(ch as u32, 0x0e30 | 0x0e32..=0x0e33 | 0x0e45..=0x0e46)
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4dbf
            | 0x4e00..=0x9fff
            | 0xf900..=0xfaff
            | 0x3040..=0x30ff
            | 0x31f0..=0x31ff
            | 0xac00..=0xd7af
    )
}

fn is_emoji(ch: char) -> bool {
    matches!(ch as u32, 0x1f000..=0x1faff | 0x2600..=0x26ff | 0x2700..=0x27bf)
}

fn is_breakable_cluster(cluster: &str) -> bool {
    cluster.chars().next().is_some_and(|ch| {
        is_thai_char(ch) || is_cjk(ch) || is_emoji(ch) || is_regional_indicator(ch)
    })
}

fn is_soft_break_punctuation(ch: char) -> bool {
    matches!(ch, '-' | '/' | '\u{2010}' | '\u{2013}' | '\u{2014}')
}

fn estimated_cluster_em(cluster: &str, monospace: bool) -> f32 {
    if cluster.chars().all(char::is_whitespace) {
        return if monospace { 0.62 } else { 0.33 };
    }

    if cluster.contains('\u{200d}')
        || cluster.chars().any(is_emoji)
        || cluster.chars().any(is_regional_indicator)
    {
        return 1.0;
    }

    cluster
        .chars()
        .filter(|ch| !is_extension(*ch) && *ch != '\u{200d}')
        .map(|ch| visible_char_em(ch, monospace))
        .sum::<f32>()
        .max(if monospace { 0.62 } else { 0.3 })
}

fn visible_char_em(ch: char, monospace: bool) -> f32 {
    if is_cjk(ch) {
        return 1.0;
    }
    if is_thai_char(ch) {
        return 0.62;
    }
    if monospace {
        return 0.62;
    }
    if ch.is_ascii_uppercase() {
        0.62
    } else if ch.is_ascii_lowercase() {
        0.52
    } else if ch.is_ascii_digit() {
        0.55
    } else if ch.is_ascii_punctuation() {
        0.36
    } else {
        0.62
    }
}

#[cfg(test)]
mod tests {
    use super::{estimated_text_width, fallback_text_width, line_break_units};

    #[test]
    fn latin_word_stays_together() {
        let units = line_break_units("CherryBrowser engine");
        assert_eq!(units[0].text, "CherryBrowser");
        assert!(units[1].is_space);
        assert_eq!(units[2].text, "engine");
    }

    #[test]
    fn thai_text_without_spaces_exposes_break_opportunities() {
        let units = line_break_units("ภาษาไทยทดสอบ");
        assert!(units.len() > 3);
        assert!(units.iter().all(|unit| !unit.is_space));
    }

    #[test]
    fn thai_marks_and_leading_vowels_stay_with_base_cluster() {
        let units = line_break_units("เก่ง");
        assert!(units.iter().any(|unit| unit.text.starts_with("เก")));
        assert!(units.iter().any(|unit| unit.text.contains("ง")));
        assert!(!units.iter().any(|unit| unit.text == "่"));
    }

    #[test]
    fn cjk_can_break_between_characters() {
        let units = line_break_units("中文測試");
        assert_eq!(units.len(), 4);
    }

    #[test]
    fn combining_marks_do_not_add_full_character_width() {
        let base = fallback_text_width("ก", 16.0, false);
        let marked = fallback_text_width("ก้", 16.0, false);
        assert!((base - marked).abs() < 0.01);
    }

    #[test]
    fn emoji_zwj_sequence_is_one_visual_cluster() {
        let width = fallback_text_width("👩\u{200d}💻", 20.0, false);
        assert!((width - 20.0).abs() < 0.01);
        assert_eq!(line_break_units("👩\u{200d}💻").len(), 1);
    }

    #[test]
    fn whitespace_is_collapsed_to_one_break_unit() {
        let units = line_break_units("a   \n\t b");
        assert_eq!(units.len(), 3);
        assert!(units[1].is_space);
        assert_eq!(units[1].text, " ");
    }

    #[test]
    fn public_width_api_uses_fallback_without_native_measurer() {
        assert!(estimated_text_width("Cherry", 16.0, false) > 0.0);
    }
}
