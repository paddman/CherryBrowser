//! Bounded, explicitly saved local workspace data. Never stores browsing history.
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::Url;

const MAX_FILE_BYTES: usize = 512 * 1024;
pub const MAX_BOOKMARKS: usize = 128;
pub const MAX_NOTE_CHARS: usize = 16_000;
const HEADER: &str = "CHERRY-WORKSPACE-1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchProvider {
    #[default]
    DuckDuckGo,
    Bing,
}

impl SearchProvider {
    pub fn label(self) -> &'static str {
        match self {
            Self::DuckDuckGo => "DuckDuckGo HTML",
            Self::Bing => "Bing",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedWorkspace {
    pub bookmarks: Vec<Bookmark>,
    pub notes: String,
    pub show_companion: bool,
    pub focus_mode: bool,
    pub search_provider: SearchProvider,
}

impl Default for SavedWorkspace {
    fn default() -> Self {
        Self {
            bookmarks: Vec::new(),
            notes: String::new(),
            show_companion: true,
            focus_mode: false,
            search_provider: SearchProvider::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Workspace {
    pub data: SavedWorkspace,
    pub dirty: bool,
    pub notice: Option<String>,
    pub filter: String,
    pub draft_title: String,
    pub draft_url: String,
    pub editing_url: Option<String>,
    path: Option<PathBuf>,
}

impl Workspace {
    pub fn load() -> Self {
        let mut workspace = Self {
            path: storage_path(),
            ..Self::default()
        };
        if let Some(path) = workspace.path.as_ref() {
            let result = File::open(path).and_then(|file| {
                let mut text = String::new();
                file.take((MAX_FILE_BYTES + 1) as u64)
                    .read_to_string(&mut text)?;
                Ok(text)
            });
            match result {
                Ok(text) => match decode(&text) {
                    Ok(data) => workspace.data = data,
                    Err(error) => {
                        workspace.notice = Some(format!(
                            "Workspace could not be read: {error}. Original file unchanged; saving disabled."
                        ));
                        workspace.path = None;
                    }
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    workspace.notice = Some(format!(
                        "Workspace unavailable: {error}. Changes are session-only."
                    ));
                    workspace.path = None;
                }
            }
        } else {
            workspace.notice = Some("No local data directory. Changes are session-only.".into());
        }
        workspace
    }

    pub fn storage_label(&self) -> String {
        self.path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Session only — persistent storage unavailable".into())
    }

    pub fn can_save(&self) -> bool {
        self.path.is_some()
    }

    pub fn save(&mut self) {
        let result = self
            .path
            .as_ref()
            .ok_or_else(|| "Local storage is unavailable".to_string())
            .and_then(|path| encode(&self.data).and_then(|text| atomic_write(path, &text)));
        match result {
            Ok(()) => {
                self.dirty = false;
                self.notice =
                    Some("Workspace saved locally. Browsing history was not saved.".into());
            }
            Err(error) => {
                self.notice = Some(format!(
                    "Save failed: {error}. Your edits remain in memory."
                ))
            }
        }
    }

    pub fn bookmark(&mut self, title: &str, url: &str) -> Result<(), String> {
        let link = make_bookmark(title, url)?;
        if let Some(existing) = self
            .data
            .bookmarks
            .iter_mut()
            .find(|item| item.url == link.url)
        {
            *existing = link;
        } else {
            if self.data.bookmarks.len() >= MAX_BOOKMARKS {
                return Err(format!("Bookmark limit reached ({MAX_BOOKMARKS})"));
            }
            self.data.bookmarks.push(link);
        }
        self.dirty = true;
        self.notice = Some("Bookmark updated. Use Save workspace to keep it after closing.".into());
        Ok(())
    }

    pub fn save_draft(&mut self) {
        let result = make_bookmark(&self.draft_title, &self.draft_url).and_then(|link| {
            if let Some(original) = self.editing_url.as_ref()
                && let Some(index) = self
                    .data
                    .bookmarks
                    .iter()
                    .position(|item| &item.url == original)
            {
                if self
                    .data
                    .bookmarks
                    .iter()
                    .enumerate()
                    .any(|(i, item)| i != index && item.url == link.url)
                {
                    return Err(
                        "That URL is already bookmarked. Edit the existing entry instead.".into(),
                    );
                }
                self.data.bookmarks[index] = link;
                self.dirty = true;
                Ok(())
            } else {
                self.bookmark(&link.title, &link.url)
            }
        });
        match result {
            Ok(()) => {
                self.draft_title.clear();
                self.draft_url.clear();
                self.editing_url = None;
                self.notice =
                    Some("Bookmark updated. Save workspace to persist your changes.".into());
            }
            Err(error) => self.notice = Some(error),
        }
    }
}

pub fn http_url(input: &str) -> Result<String, String> {
    if input.len() > 4096 {
        return Err("URL is too long (maximum 4096 bytes)".into());
    }
    let normalized = crate::net::normalize_url(input)?;
    let url = Url::parse(&normalized).map_err(|error| error.to_string())?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(
            "URLs containing credentials are not accepted. Use a URL without user:password@."
                .into(),
        );
    }
    if url.host_str().is_none() {
        return Err("URL must have a host".into());
    }
    Ok(url.to_string())
}

/// Classify submitted input only. Typing never sends suggestions to a server.
pub fn navigation_target(input: &str, provider: SearchProvider) -> Result<String, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Enter a URL or search terms".into());
    }
    if input.len() > 4096 {
        return Err("Input is too long (maximum 4096 bytes)".into());
    }
    let authority = input.split(['/', '?', '#']).next().unwrap_or(input);
    let host_port = authority.rsplit_once(':').is_some_and(|(host, port)| {
        !host.is_empty() && !port.is_empty() && port.chars().all(|c| c.is_ascii_digit())
    });
    if input.contains("://") {
        return http_url(input);
    }
    if let Some((scheme, _)) = input.split_once(':')
        && !host_port
        && !scheme.contains(char::is_whitespace)
        && !scheme.contains('/')
    {
        return Err(format!("Unsupported or malformed URL scheme: {scheme}"));
    }
    let looks_like_host = !input.contains(char::is_whitespace)
        && (authority.contains('.')
            || authority == "localhost"
            || host_port
            || authority.starts_with('['));
    if looks_like_host {
        return http_url(input);
    }
    let endpoint = match provider {
        SearchProvider::DuckDuckGo => "https://html.duckduckgo.com/html/",
        SearchProvider::Bing => "https://www.bing.com/search",
    };
    let mut url = Url::parse(endpoint).map_err(|error| error.to_string())?;
    url.query_pairs_mut().append_pair("q", input);
    Ok(url.to_string())
}

fn make_bookmark(title: &str, url: &str) -> Result<Bookmark, String> {
    let url = http_url(url)?;
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = if title.is_empty() { url.clone() } else { title };
    Ok(Bookmark {
        title: title.chars().take(120).collect(),
        url,
    })
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape(text: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        result.push(match chars.next() {
            Some('\\') => '\\',
            Some('t') => '\t',
            Some('n') => '\n',
            Some('r') => '\r',
            _ => return Err("Invalid workspace escape sequence".into()),
        });
    }
    Ok(result)
}

fn encode(data: &SavedWorkspace) -> Result<String, String> {
    if data.bookmarks.len() > MAX_BOOKMARKS || data.notes.chars().count() > MAX_NOTE_CHARS {
        return Err("Workspace exceeds its safety limits".into());
    }
    let provider = match data.search_provider {
        SearchProvider::DuckDuckGo => "duckduckgo",
        SearchProvider::Bing => "bing",
    };
    let mut text = format!(
        "{HEADER}\ncompanion\t{}\nfocus\t{}\nsearch\t{provider}\nnotes\t{}\n",
        u8::from(data.show_companion),
        u8::from(data.focus_mode),
        escape(&data.notes)
    );
    for link in &data.bookmarks {
        let link = make_bookmark(&link.title, &link.url)?;
        text.push_str(&format!(
            "site\t{}\t{}\n",
            escape(&link.title),
            escape(&link.url)
        ));
    }
    if text.len() > MAX_FILE_BYTES {
        return Err("Workspace file is too large".into());
    }
    Ok(text)
}

fn decode(text: &str) -> Result<SavedWorkspace, String> {
    if text.len() > MAX_FILE_BYTES {
        return Err("Workspace file is too large".into());
    }
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err("Unknown workspace format".into());
    }
    let mut data = SavedWorkspace::default();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        match fields.as_slice() {
            ["companion", value] => data.show_companion = parse_bool(value)?,
            ["focus", value] => data.focus_mode = parse_bool(value)?,
            ["search", "duckduckgo"] => data.search_provider = SearchProvider::DuckDuckGo,
            ["search", "bing"] => data.search_provider = SearchProvider::Bing,
            ["notes", value] => {
                data.notes = unescape(value)?;
                if data.notes.chars().count() > MAX_NOTE_CHARS {
                    return Err("Notes exceed the character limit".into());
                }
            }
            ["site", title, url] => {
                let link = make_bookmark(&unescape(title)?, &unescape(url)?)?;
                if data.bookmarks.iter().any(|item| item.url == link.url) {
                    continue;
                }
                if data.bookmarks.len() >= MAX_BOOKMARKS {
                    return Err("Too many bookmarks".into());
                }
                data.bookmarks.push(link);
            }
            _ => return Err("Malformed workspace record".into()),
        }
    }
    Ok(data)
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err("Invalid workspace setting".into()),
    }
}

fn storage_path() -> Option<PathBuf> {
    let home = || {
        std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        home().map(|path| path.join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home().map(|path| path.join(".local/share")))
    };
    base.map(|path| path.join("CherryBrowser/workspace.txt"))
}

fn atomic_write(path: &Path, text: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid workspace path")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let temporary = parent.join(format!(".workspace-{}-{nonce}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // A complete temporary file is written before the destination is replaced.
    let mut file = options
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let result = (|| {
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_encodes_unicode_and_query_delimiters() {
        let target =
            navigation_target("ภาษาไทย Rust & a=b #test", SearchProvider::DuckDuckGo).unwrap();
        let url = Url::parse(&target).unwrap();
        assert_eq!(url.host_str(), Some("html.duckduckgo.com"));
        assert_eq!(
            url.query_pairs().collect::<Vec<_>>(),
            vec![("q".into(), "ภาษาไทย Rust & a=b #test".into())]
        );
        assert!(url.fragment().is_none());
    }

    #[test]
    fn url_and_host_port_are_not_search_queries() {
        for input in [
            "example.com",
            "https://example.com/a?q=x",
            "localhost:8080",
            "[::1]:8080",
        ] {
            assert!(
                !navigation_target(input, SearchProvider::Bing)
                    .unwrap()
                    .contains("bing.com")
            );
        }
    }

    #[test]
    fn rejects_unsafe_schemes_and_credentials() {
        for input in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,x",
            "https://user:pass@example.com",
        ] {
            assert!(navigation_target(input, SearchProvider::DuckDuckGo).is_err());
        }
    }

    #[test]
    fn empty_and_oversized_queries_fail() {
        assert!(navigation_target("  ", SearchProvider::Bing).is_err());
        assert!(navigation_target(&"a".repeat(4097), SearchProvider::Bing).is_err());
    }

    #[test]
    fn workspace_round_trip_preserves_unicode_and_escapes() {
        let mut data = SavedWorkspace {
            notes: "ไทย\nnext\tline\\path\r🙂".into(),
            focus_mode: true,
            search_provider: SearchProvider::Bing,
            ..SavedWorkspace::default()
        };
        data.bookmarks
            .push(make_bookmark("Rust ภาษาไทย", "https://example.com/?a=1&b=2").unwrap());
        assert_eq!(decode(&encode(&data).unwrap()).unwrap(), data);
    }

    #[test]
    fn malformed_data_is_rejected_not_silently_replaced() {
        for text in [
            "bad header",
            "CHERRY-WORKSPACE-1\nfocus\tmaybe",
            "CHERRY-WORKSPACE-1\nnotes\tbad\\x",
            "CHERRY-WORKSPACE-1\nsite\tX\tfile:///tmp/x",
        ] {
            assert!(decode(text).is_err());
        }
    }

    #[test]
    fn bookmark_updates_are_deduplicated() {
        let mut workspace = Workspace::default();
        workspace.bookmark("First", "example.com").unwrap();
        workspace
            .bookmark("Updated", "https://example.com/")
            .unwrap();
        assert_eq!(workspace.data.bookmarks.len(), 1);
        assert_eq!(workspace.data.bookmarks[0].title, "Updated");
        assert!(workspace.dirty);
    }

    #[test]
    fn editing_changes_the_original_entry() {
        let mut workspace = Workspace::default();
        workspace.bookmark("First", "example.com").unwrap();
        workspace.editing_url = Some("https://example.com/".into());
        workspace.draft_url = "https://example.org".into();
        workspace.draft_title = "Updated".into();
        workspace.save_draft();
        assert_eq!(workspace.data.bookmarks.len(), 1);
        assert_eq!(workspace.data.bookmarks[0].url, "https://example.org/");
        assert!(workspace.editing_url.is_none());
    }

    #[test]
    fn notes_and_bookmarks_have_limits() {
        let data = SavedWorkspace {
            notes: "x".repeat(MAX_NOTE_CHARS + 1),
            ..SavedWorkspace::default()
        };
        assert!(encode(&data).is_err());
        let mut workspace = Workspace::default();
        for i in 0..MAX_BOOKMARKS {
            workspace
                .bookmark("Site", &format!("https://example.com/{i}"))
                .unwrap();
        }
        assert!(
            workspace
                .bookmark("Overflow", "https://example.org")
                .is_err()
        );
    }

    #[test]
    fn atomic_save_replaces_a_complete_file() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "cherry-workspace-test-{}-{nonce}",
            std::process::id()
        ));
        let path = directory.join("workspace.txt");
        atomic_write(&path, "first").unwrap();
        atomic_write(&path, "second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
}
