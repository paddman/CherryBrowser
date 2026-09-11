use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
    thread,
};

use crate::{
    dom::{Dom, NodeId},
    html, image_data,
    image_data::DecodedImage,
    net::{self, FetchResponse},
};

const MAX_EXTERNAL_STYLESHEETS: usize = 24;
const MAX_DOCUMENT_IMAGES: usize = 32;
const MAX_RESOURCE_WORKERS: usize = 6;

#[derive(Debug, Clone)]
pub struct LoadedDocument {
    pub response: FetchResponse,
    pub dom: Option<Dom>,
    pub base_url: String,
    pub stylesheet_source: String,
    pub external_stylesheets: usize,
    pub images: HashMap<NodeId, DecodedImage>,
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
            images: HashMap::new(),
            resource_warnings: Vec::new(),
        });
    }

    let dom = html::parse(&response.body);
    let base_url = document_base_url(&dom, &final_url);
    let mut resource_warnings = Vec::new();

    let stylesheet_urls = stylesheet_links(&dom, &base_url, &mut resource_warnings);
    let image_urls = image_links(&dom, &base_url, &mut resource_warnings);

    let (external, stylesheet_warnings) = fetch_stylesheets_parallel(&stylesheet_urls);
    let (images, image_warnings) = fetch_images_parallel(&image_urls);
    resource_warnings.extend(stylesheet_warnings);
    resource_warnings.extend(image_warnings);

    let stylesheet_source = compose_stylesheet_source(&dom, &base_url, &external);

    Ok(LoadedDocument {
        response,
        dom: Some(dom),
        base_url,
        stylesheet_source,
        external_stylesheets: external.len(),
        images,
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

fn image_links(dom: &Dom, base_url: &str, warnings: &mut Vec<String>) -> Vec<(NodeId, String)> {
    let mut links = Vec::new();

    for (id, _) in dom.nodes().iter().enumerate() {
        if dom.tag_name(id) != Some("img") {
            continue;
        }
        let Some(src) = dom.attr(id, "src") else {
            continue;
        };
        if src.trim().is_empty() {
            continue;
        }

        if links.len() >= MAX_DOCUMENT_IMAGES {
            warnings.push(format!(
                "image limit reached; ignoring resources after {MAX_DOCUMENT_IMAGES} images"
            ));
            break;
        }

        match net::resolve_url(base_url, src) {
            Ok(url) => links.push((id, url)),
            Err(error) => warnings.push(format!("image URL rejected: {error}")),
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
    let mut loaded = HashMap::new();
    let mut warnings = Vec::new();

    for chunk in urls.chunks(MAX_RESOURCE_WORKERS) {
        let (sender, receiver) = mpsc::channel();

        thread::scope(|scope| {
            for url in chunk {
                let sender = sender.clone();
                let url = url.clone();
                scope.spawn(move || {
                    let result = net::fetch_stylesheet(&url);
                    let _ = sender.send((url, result));
                });
            }
            drop(sender);
        });

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
    }

    (loaded, warnings)
}

fn fetch_images_parallel(
    urls: &[(NodeId, String)],
) -> (HashMap<NodeId, DecodedImage>, Vec<String>) {
    let jobs = group_image_jobs(urls);
    let mut loaded = HashMap::new();
    let mut warnings = Vec::new();

    for chunk in jobs.chunks(MAX_RESOURCE_WORKERS) {
        let (sender, receiver) = mpsc::channel();

        thread::scope(|scope| {
            for (url, nodes) in chunk {
                let sender = sender.clone();
                let url = url.clone();
                let nodes = nodes.clone();
                scope.spawn(move || {
                    let result = net::fetch_image(&url).and_then(|response| {
                        if !(200..300).contains(&response.status) {
                            return Err(format!("HTTP {}", response.status));
                        }
                        image_data::decode(&response.body)
                    });
                    let _ = sender.send((nodes, url, result));
                });
            }
            drop(sender);
        });

        for (nodes, url, result) in receiver {
            match result {
                Ok(image) => {
                    for node in nodes {
                        loaded.insert(node, image.clone());
                    }
                }
                Err(error) => warnings.push(format!("image failed: {url}: {error}")),
            }
        }
    }

    (loaded, warnings)
}

fn group_image_jobs(urls: &[(NodeId, String)]) -> Vec<(String, Vec<NodeId>)> {
    let mut grouped = HashMap::<String, Vec<NodeId>>::new();
    for (node, url) in urls {
        grouped.entry(url.clone()).or_default().push(*node);
    }
    grouped.into_iter().collect()
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

    use super::{
        compose_stylesheet_source, document_base_url, group_image_jobs, image_links,
        stylesheet_links,
    };

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
    fn resolves_image_urls_against_document_base() {
        let dom = html::parse(
            r#"<base href="https://img.example/static/"><img src="hero.png"><img src="icons/a.webp">"#,
        );
        let base = document_base_url(&dom, "https://example.com/");
        let mut warnings = Vec::new();
        let links = image_links(&dom, &base, &mut warnings);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].1, "https://img.example/static/hero.png");
        assert_eq!(links[1].1, "https://img.example/static/icons/a.webp");
        assert!(warnings.is_empty());
    }

    #[test]
    fn groups_duplicate_image_urls_into_one_fetch_job() {
        let jobs = group_image_jobs(&[
            (1, "https://img.example/a.png".to_string()),
            (2, "https://img.example/a.png".to_string()),
            (3, "https://img.example/b.png".to_string()),
        ]);
        assert_eq!(jobs.len(), 2);

        let shared = jobs
            .iter()
            .find(|(url, _)| url.ends_with("/a.png"))
            .unwrap();
        assert_eq!(shared.1.len(), 2);
        assert!(shared.1.contains(&1));
        assert!(shared.1.contains(&2));
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
