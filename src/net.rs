use std::time::Duration;

use reqwest::{
    Url,
    blocking::Client,
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE},
    redirect::Policy,
};

const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_STYLESHEET_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub requested_url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: String,
    pub body: String,
}

pub fn normalize_url(input: &str) -> Result<String, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("URL is empty".to_string());
    }

    let candidate = if input.contains("://") {
        input.to_string()
    } else {
        format!("https://{input}")
    };

    let parsed = Url::parse(&candidate).map_err(|error| format!("invalid URL: {error}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed.to_string()),
        scheme => Err(format!("unsupported URL scheme: {scheme}")),
    }
}

pub fn resolve_url(base: &str, target: &str) -> Result<String, String> {
    let base = Url::parse(base).map_err(|error| format!("invalid base URL: {error}"))?;
    base.join(target)
        .map(|url| url.to_string())
        .map_err(|error| format!("invalid link: {error}"))
}

pub fn fetch(url: &str) -> Result<FetchResponse, String> {
    fetch_with_limit(
        url,
        "text/html,application/xhtml+xml,text/plain;q=0.9,*/*;q=0.5",
        MAX_DOCUMENT_BYTES,
    )
}

pub fn fetch_stylesheet(url: &str) -> Result<FetchResponse, String> {
    fetch_with_limit(url, "text/css,*/*;q=0.1", MAX_STYLESHEET_BYTES)
}

fn fetch_with_limit(url: &str, accept: &str, max_bytes: usize) -> Result<FetchResponse, String> {
    let normalized = normalize_url(url)?;
    let client = Client::builder()
        .user_agent("CherryBrowser/0.2 (+https://github.com/paddman/CherryBrowser)")
        .timeout(Duration::from_secs(25))
        .connect_timeout(Duration::from_secs(10))
        .redirect(Policy::limited(10))
        .build()
        .map_err(|error| format!("failed to initialize HTTP client: {error}"))?;

    let response = client
        .get(&normalized)
        .header(ACCEPT, accept)
        .header(ACCEPT_LANGUAGE, "th,en-US;q=0.9,en;q=0.8")
        .send()
        .map_err(|error| format!("network error: {error}"))?;

    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(format!(
            "resource is larger than the current {} MiB safety limit",
            max_bytes / 1024 / 1024
        ));
    }

    let requested_url = normalized;
    let final_url = response.url().to_string();
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = response
        .bytes()
        .map_err(|error| format!("failed reading response body: {error}"))?;
    if bytes.len() > max_bytes {
        return Err(format!(
            "resource exceeded the current {} MiB safety limit",
            max_bytes / 1024 / 1024
        ));
    }

    let body = String::from_utf8_lossy(&bytes).into_owned();

    Ok(FetchResponse {
        requested_url,
        final_url,
        status,
        content_type,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::{normalize_url, resolve_url};

    #[test]
    fn normalizes_bare_hosts() {
        assert_eq!(
            normalize_url("example.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn resolves_relative_links() {
        assert_eq!(
            resolve_url("https://example.com/a/page.html", "../b").unwrap(),
            "https://example.com/b"
        );
    }

    #[test]
    fn rejects_non_web_schemes() {
        assert!(normalize_url("file:///etc/passwd").is_err());
    }
}
