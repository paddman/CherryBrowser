use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

use eframe::egui::{
    self, FontData, FontFamily,
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};

const MAX_DISCOVERED_FILES: usize = 512;
const MAX_INSTALLED_FONTS: usize = 8;
const MAX_FONT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_TOTAL_FONT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SCAN_DEPTH: usize = 4;

pub(crate) fn install_system_fallbacks(ctx: &egui::Context) -> Vec<PathBuf> {
    let mut candidates = discover_candidates();
    candidates.sort_by(|a, b| {
        font_score(b)
            .cmp(&font_score(a))
            .then_with(|| a.as_os_str().cmp(b.as_os_str()))
    });

    let mut installed = Vec::new();
    let mut total_bytes = 0_u64;

    for path in candidates {
        if installed.len() >= MAX_INSTALLED_FONTS {
            break;
        }

        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        let size = metadata.len();
        if size == 0 || size > MAX_FONT_BYTES || total_bytes.saturating_add(size) > MAX_TOTAL_FONT_BYTES {
            continue;
        }

        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let name = format!(
            "cherry-system-{}-{}",
            installed.len(),
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("font")
        );

        ctx.add_font(FontInsert {
            name,
            data: FontData::from_owned(bytes),
            families: vec![
                InsertFontFamily {
                    family: FontFamily::Proportional,
                    priority: FontPriority::Lowest,
                },
                InsertFontFamily {
                    family: FontFamily::Monospace,
                    priority: FontPriority::Lowest,
                },
            ],
        });

        total_bytes += size;
        installed.push(path);
    }

    installed
}

fn discover_candidates() -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for root in font_roots() {
        scan_directory(&root, 0, &mut seen, &mut candidates);
        if candidates.len() >= MAX_DISCOVERED_FILES {
            break;
        }
    }

    candidates
}

fn scan_directory(
    path: &Path,
    depth: usize,
    seen: &mut HashSet<PathBuf>,
    candidates: &mut Vec<PathBuf>,
) {
    if depth > MAX_SCAN_DEPTH || candidates.len() >= MAX_DISCOVERED_FILES {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        if candidates.len() >= MAX_DISCOVERED_FILES {
            return;
        }
        let path = entry.path();
        if path.is_dir() {
            scan_directory(&path, depth + 1, seen, candidates);
            continue;
        }
        if !is_font_file(&path) || font_score(&path) == 0 {
            continue;
        }

        let canonical = fs::canonicalize(&path).unwrap_or(path);
        if seen.insert(canonical.clone()) {
            candidates.push(canonical);
        }
    }
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc"))
}

fn font_score(path: &Path) -> u16 {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mut score = 0;
    for (needle, weight) in [
        ("notosansthai", 120),
        ("notoserifthai", 115),
        ("leelaw", 110),
        ("tahoma", 105),
        ("loma", 100),
        ("garuda", 95),
        ("kinnari", 90),
        ("norasi", 90),
        ("waree", 90),
        ("tlwg", 85),
        ("notosanscjk", 80),
        ("sourcehan", 78),
        ("pingfang", 76),
        ("meiryo", 74),
        ("yahei", 72),
        ("malgun", 70),
        ("notocoloremoji", 65),
        ("seguiemj", 64),
        ("emoji", 60),
        ("unicode", 50),
    ] {
        if name.contains(needle) {
            score = score.max(weight);
        }
    }
    score
}

fn font_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let windows = env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        roots.push(windows.join("Fonts"));
    }

    #[cfg(target_os = "linux")]
    {
        roots.push(PathBuf::from("/usr/share/fonts"));
        roots.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            roots.push(home.join(".fonts"));
            roots.push(home.join(".local/share/fonts"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        roots.push(PathBuf::from("/System/Library/Fonts"));
        roots.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            roots.push(home.join("Library/Fonts"));
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = &mut roots;
        let _ = env::var_os("HOME");
    }

    roots
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{font_score, is_font_file};

    #[test]
    fn recognizes_supported_font_file_extensions() {
        assert!(is_font_file(Path::new("NotoSansThai-Regular.ttf")));
        assert!(is_font_file(Path::new("PingFang.ttc")));
        assert!(!is_font_file(Path::new("font.woff2")));
    }

    #[test]
    fn prioritizes_thai_fonts_above_generic_unicode_fallbacks() {
        assert!(
            font_score(Path::new("NotoSansThai-Regular.ttf"))
                > font_score(Path::new("ArialUnicode.ttf"))
        );
        assert_eq!(font_score(Path::new("ordinary-latin.ttf")), 0);
    }
}
