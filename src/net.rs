use std::{io::Read, sync::OnceLock, time::Duration};

use reqwest::{
    Url,
    blocking::Client,
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE},
    redirect::Policy,
};

use crate::{cancel::CancellationToken, text};

const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_STYLESHEET_BYTES: usize = 2 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 12 * 1024 * 1024;
const STREAM_CHUNK_BYTES: usize = 16 * 1024;

static HTTP_CLIENT: OnceLock<Result<Client, String>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub requested_url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct BinaryFetchResponse {
    pub requested_url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
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
    fetch_with_cancel(url, &CancellationToken::new())
}

pub fn fetch_with_cancel(url: &str, cancel: &CancellationToken) -> Result<FetchResponse, String> {
    fetch_text_with_limit(
        url,
        "text/html,application/xhtml+xml,text/plain;q=0.9,*/*;q=0.5",
        MAX_DOCUMENT_BYTES,
        true,
        cancel,
    )
}

pub fn fetch_stylesheet(url: &str) -> Result<FetchResponse, String> {
    fetch_stylesheet_with_cancel(url, &CancellationToken::new())
}

pub fn fetch_stylesheet_with_cancel(
    url: &str,
    cancel: &CancellationToken,
) -> Result<FetchResponse, String> {
    fetch_text_with_limit(
        url,
        "text/css,*/*;q=0.1",
        MAX_STYLESHEET_BYTES,
        false,
        cancel,
    )
}

pub fn fetch_image(url: &str) -> Result<BinaryFetchResponse, String> {
    fetch_image_with_cancel(url, &CancellationToken::new())
}

pub fn fetch_image_with_cancel(
    url: &str,
    cancel: &CancellationToken,
) -> Result<BinaryFetchResponse, String> {
    fetch_bytes_with_limit(
        url,
        "image/avif,image/webp,image/png,image/jpeg,image/*;q=0.8,*/*;q=0.1",
        MAX_IMAGE_BYTES,
        cancel,
    )
}

fn http_client() -> Result<&'static Client, String> {
    HTTP_CLIENT
        .get_or_init(|| {
            Client::builder()
                .user_agent("CherryBrowser/0.3.1 (+https://github.com/paddman/CherryBrowser)")
                .timeout(Duration::from_secs(25))
                .connect_timeout(Duration::from_secs(10))
                .redirect(Policy::limited(10))
                .build()
                .map_err(|error| format!("failed to initialize HTTP client: {error}"))
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn fetch_text_with_limit(
    url: &str,
    accept: &str,
    max_bytes: usize,
    sniff_html: bool,
    cancel: &CancellationToken,
) -> Result<FetchResponse, String> {
    let response = fetch_bytes_with_limit(url, accept, max_bytes, cancel)?;
    let body = text::decode_web_text(&response.body, &response.content_type, sniff_html);

    Ok(FetchResponse {
        requested_url: response.requested_url,
        final_url: response.final_url,
        status: response.status,
        content_type: response.content_type,
        body,
    })
}

fn fetch_bytes_with_limit(
    url: &str,
    accept: &str,
    max_bytes: usize,
    cancel: &CancellationToken,
) -> Result<BinaryFetchResponse, String> {
    cancel.check()?;
    let normalized = normalize_url(url)?;
    let mut response = http_client()?
        .get(&normalized)
        .header(ACCEPT, accept)
        .header(ACCEPT_LANGUAGE, "th,en-US;q=0.9,en;q=0.8")
        .send()
        .map_err(|error| format!("network error: {error}"))?;
    cancel.check()?;

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
    let body = read_body_with_limit(&mut response, max_bytes, cancel)?;

    Ok(BinaryFetchResponse {
        requested_url,
        final_url,
        status,
        content_type,
        body,
    })
}

fn read_body_with_limit(
    reader: &mut impl Read,
    max_bytes: usize,
    cancel: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(max_bytes.min(64 * 1024));
    let mut reader = reader.take(max_bytes as u64 + 1);
    let mut chunk = [0_u8; STREAM_CHUNK_BYTES];

    loop {
        cancel.check()?;
        let read = reader
            .read(&mut chunk)
            .map_err(|error| format!("failed reading response body: {error}"))?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        if bytes.len() > max_bytes {
            return Err(format!(
                "resource exceeded the current {} MiB safety limit",
                max_bytes / 1024 / 1024
            ));
        }
    }

    cancel.check()?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::cancel::CancellationToken;

    use super::{normalize_url, read_body_with_limit, resolve_url};

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

    #[test]
    fn stops_streaming_after_resource_limit() {
        let mut reader = Cursor::new(vec![7_u8; 33]);
        let token = CancellationToken::new();
        let error = read_body_with_limit(&mut reader, 32, &token).unwrap_err();
        assert!(error.contains("resource exceeded"));
        assert_eq!(reader.position(), 33);
    }

    #[test]
    fn returns_bounded_stream_body() {
        let mut reader = Cursor::new(b"cherry".to_vec());
        let token = CancellationToken::new();
        assert_eq!(
            read_body_with_limit(&mut reader, 32, &token).unwrap(),
            b"cherry"
        );
    }

    #[test]
    fn cancelled_stream_stops_before_reading() {
        let mut reader = Cursor::new(b"cherry".to_vec());
        let token = CancellationToken::new();
        token.cancel();
        let error = read_body_with_limit(&mut reader, 32, &token).unwrap_err();
        assert!(error.contains("cancelled"));
        assert_eq!(reader.position(), 0);
    }
}
