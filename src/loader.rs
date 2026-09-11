use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
    thread,
};

use crate::{
    dom::Dom,
    html,
    net::{self, FetchResponse},
};

const MAX_EXTERNAL_STYLESHEETS: usize = 24;

#[derive(Debug, Clone)]
pub struct LoadedDocument {
    pub response: FetchResponse,
    pub dom: Option<Dom>,
    pub base_url: String,
    pub stylesheet_source: String,
    pub external_stylesheets: usize,
    pub resource_warnings: Vec<String>,
}

pub fn load(url: &str) -> Result<LoadedDocument, String> {
    let response = net::fetch(url)?;
    let final_url = response.final_url.clone();
    if !is_html(&response.content_type) {
        return Ok(LoadedDocument {
            response,
            dom: None,
            base_url: final_url,
            stylesheet_source: String::new(),
            external_stylesheets: 0,
            resource_warnings: Vec::new(),
        });
    }

    let dom = html::parse(&response.body);
    let base_url = document_base_url(&dom, &final_url);
    let mut resource_warnings = Vec::new();
    let links = stylesheet_links(&dom, &base_url, &mut resource_warnings);
    let (external, fetch_warnings) = fetch_stylesheets_parallel(&links);
    resource_warnings.extend(fetch_warnings);

    let stylesheet_source = compose_stylesheet_source(&dom, &base_url, &external);

    Ok(LoadedDocument {
        response,
        dom: Some(dom),
        base_url,
        stylesheet_source,
        external_stylesheets: external.len(),
        resource_warnings,
    })
}

fn is_html(content_type: &str) -> bool {
    let content_type = content_type.to_ascii_lowercase();
    content_type.contains("text/html") || content_type.contains("application/xhtml+xml")
}

fn document_base_url(dom: &Dom, fallback: &str) -> String {
    for (id, _) in dom.nodes().iter().enumerate() {
        if dom.tag_name(id) != Some("base") {
            continue;
        }
        let Some(href) = dom.attr(id, "href") else {
            continue;
        };
        if let Ok(resolved) = net::resolve_url(fallback, href) {
            return resolved;
        }
    }
    fallback.to_string()
}

fn stylesheet_links(dom: &Dom, base_url: &str, warnings: &mut Vec<String>) -> Vec<String> {
    let mut links = Vec::new();
    let mut seen = HashSet::new();

    for (id, _) in dom.nodes().iter().enumerate() {
        if dom.tag_name(id) != Some("link") || !is_stylesheet_link(dom, id) {
            continue;
        }

        let Some(href) = dom.attr(id, "href") else {
            continue;
        };
        match net::resolve_url(base_url, href) {
            Ok(url) => {
                if seen.insert(url.clone()) {
                    if links.len() >= MAX_EXTERNAL_STYLESHEETS {
                        warnings.push(format!(
                            "stylesheet limit reached; ignoring resources after {MAX_EXTERNAL_STYLESHEETS} files"
                        ));
                        break;
                    }
                    links.push(url);
                }
            }
            Err(error) => warnings.push(format!("stylesheet URL rejected: {error}")),
        }
    }

    links
}

fn is_stylesheet_link(dom: &Dom, id: usize) -> bool {
    if dom.attr(id, "disabled").is_some() {
        return false;
    }

    let rel = dom.attr(id, "rel").unwrap_or("");
    if !rel
        .split_ascii_whitespace()
        .any(|token| token.eq_ignore_ascii_case("stylesheet"))
    {
        return false;
    }

    if let Some(kind) = dom.attr(id, "type")
        && !kind.is_empty()
        && !kind.eq_ignore_ascii_case("text/css")
    {
        return false;
    }

    match dom.attr(id, "media") {
        None | Some("") => true,
        Some(media) => media.split(',').any(|query| {
            let query = query.trim().to_ascii_lowercase();
            !query.starts_with("not ")
                && (query == "all" || query == "screen" || query.contains("screen"))
        }),
    }
}

fn fetch_stylesheets_parallel(urls: &[String]) -> (HashMap<String, String>, Vec<String>) {
    let (sender, receiver) = mpsc::channel();

    thread::scope(|scope| {
        for url in urls {
            let sender = sender.clone();
            let url = url.clone();
            scope.spawn(move || {
                let result = net::fetch_stylesheet(&url);
                let _ = sender.send((url, result));
            });
        }
        drop(sender);
    });

    let mut loaded = HashMap::new();
    let mut warnings = Vec::new();

    for (requested_url, result) in receiver {
        match result {
            Ok(response) if (200..300).contains(&response.status) => {
                loaded.insert(requested_url, response.body);
            }
            Ok(response) => warnings.push(format!(
                "stylesheet returned HTTP {}: {}",
                response.status, requested_url
            )),
            Err(error) => warnings.push(format!("stylesheet failed: {requested_url}: {error}")),
        }
    }

    (loaded, warnings)
}

fn compose_stylesheet_source(
    dom: &Dom,
    base_url: &str,
    external: &HashMap<String, String>,
) -> String {
    let mut out = String::new();

    for (id, _) in dom.nodes().iter().enumerate() {
        match dom.tag_name(id) {
            Some("style") => {
                out.push_str(&dom.text_content(id));
                out.push('\n');
            }
            Some("link") if is_stylesheet_link(dom, id) => {
                let Some(href) = dom.attr(id, "href") else {
                    continue;
                };
                let Ok(url) = net::resolve_url(base_url, href) else {
                    continue;
                };
                if let Some(css) = external.get(&url) {
                    out.push_str(css);
                    out.push('\n');
                }
            }
            _ => {}
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::html;

    use super::{compose_stylesheet_source, document_base_url, stylesheet_links};

    #[test]
    fn base_href_changes_resource_resolution() {
        let dom = html::parse(
            r#"<html><head><base href="https://cdn.example/assets/"><link rel="stylesheet" href="app.css"></head></html>"#,
        );
        let base = document_base_url(&dom, "https://example.com/docs/index.html");
        assert_eq!(base, "https://cdn.example/assets/");

        let mut warnings = Vec::new();
        let links = stylesheet_links(&dom, &base, &mut warnings);
        assert_eq!(links, vec!["https://cdn.example/assets/app.css"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn composes_embedded_and_external_css_in_dom_order() {
        let dom = html::parse(
            r#"<style>p{color:red}</style><link rel="stylesheet" href="x.css"><style>p{color:green}</style>"#,
        );
        let mut external = HashMap::new();
        external.insert(
            "https://example.com/x.css".to_string(),
            "p{color:blue}".to_string(),
        );

        let source = compose_stylesheet_source(&dom, "https://example.com/", &external);
        assert!(source.find("red").unwrap() < source.find("blue").unwrap());
        assert!(source.find("blue").unwrap() < source.find("green").unwrap());
    }
}
