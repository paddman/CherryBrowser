# Cherry Engine Architecture

## Boundary

CherryBrowser owns the web-platform-facing engine layers:

- HTML parsing and DOM construction
- CSS parsing, selector matching and cascade
- document/resource orchestration and web URL policy
- web text decoding policy
- navigation cancellation policy
- style computation
- layout
- display-list generation
- image placement and page-level resource identity
- hit testing
- navigation/history policy
- future JavaScript runtime and DOM bindings
- future compositor and process isolation

Generic infrastructure libraries are allowed for TLS, HTTP framing, cryptography, OS windows, graphics APIs, image codecs, fonts and compression.

The engine must not embed or delegate page semantics to Chromium/Blink, WebKit, Firefox/Gecko, CEF, Electron, a system WebView, or another browser renderer.

## Current 0.3.1 pipeline

```text
                       +-------------------------+
URL ------------------>| Cherry network policy   |
                       | HTTP/HTTPS only          |
                       +------------+------------+
                                    |
                                    v
                       +-------------------------+
                       | shared reqwest transport|
                       | pooled HTTP/TLS conns   |
                       +------------+------------+
                                    |
                                    v
                       +-------------------------+
                       | bounded stream reader   |
                       | + cancellation checks   |
                       +------------+------------+
                                    |
                                    v
                       +-------------------------+
                       | Cherry text decoder     |
                       | BOM/header/meta charset |
                       +------------+------------+
                                    |
                                    v
                       +-------------------------+
                       | Cherry document loader  |
                       +----+---------------+----+
                            |               |
                +-----------+               +----------------+
                |                                            |
                v                                            v
      bounded CSS workers                          bounded image workers
                |                                  + URL deduplication
                v                                            v
       ordered CSS source                        bounded image decoder
                |                                   PNG/JPEG/WebP
                |                                            |
                +------------------+   +---------------------+
                                   |   |
                                   v   v
                            +----------------+
                            | Cherry DOM     |
                            +-------+--------+
                                    |
                                    v
                            +----------------+
                            | CSS cascade    |
                            +-------+--------+
                                    |
                                    v
                            +----------------+
                            | Cherry layout  |
                            +-------+--------+
                                    |
                                    v
                            +----------------+
                            | display list   |
                            +-------+--------+
                                    |
                                    v
                            +----------------+
                            | native painter |
                            | + texture cache|
                            +----------------+
```

The DOM is parsed once per HTML navigation and is reused for resource discovery, style, layout, and link resolution. `<base href>` is carried as page state so resource URLs and clicked relative links resolve consistently.

Navigation owns a cooperative cancellation token. Starting a newer navigation cancels the previous token. The network reader checks cancellation before a request, after response headers, and between streamed body chunks. The loader also checks cancellation between parsing and resource phases, and subresource workers share the same token. Blocking DNS/connect/TLS work may still return only when the underlying blocking operation yields, so this is cooperative cancellation rather than an OS-level hard abort.

Text responses no longer assume UTF-8 unconditionally. The current decoder gives BOM precedence, then HTTP `charset`, then the first 1024 bytes of HTML for `<meta charset>`/charset declarations. Initial legacy support includes web-compatible Windows-1252 labels and Windows-874/TIS-620 for Thai pages. Valid undeclared UTF-8 is preserved; malformed undeclared input currently falls back to Windows-1252.

The image codec only converts bounded PNG/JPEG/WebP bytes into RGBA pixels. Cherry code decides which `<img>` resources exist, resolves URLs, deduplicates repeated image URLs, enforces fetch budgets, chooses layout geometry, emits image paint items, owns texture identity, and checks final RGBA allocation size.

The egui/Glow layer is still a temporary native presentation shell. DOM/CSS semantics, layout coordinates, display-list items and hit regions are generated by Cherry code, so the shell can later be replaced by a dedicated wgpu/Vulkan/Metal/D3D compositor without replacing the browser engine.

## Current isolation and safety budgets

Milestone 0.3.1 remains single-process, so untrusted web content is bounded aggressively:

- documents: 8 MiB
- external CSS: 2 MiB each, maximum 24
- images: 12 MiB compressed each, maximum 32
- CSS/image resource concurrency: maximum 6 workers per batch
- decoded image dimensions: 8192 × 8192 maximum
- decoder/RGBA allocation budget: 128 MiB per image
- redirects: maximum 10
- network timeout: 25 seconds
- connect timeout: 10 seconds
- URL schemes: HTTP and HTTPS only
- superseded navigation: cooperative cancellation across loader/network/subresources

These limits are not a replacement for renderer sandboxing. They reduce accidental and deliberate resource abuse until the process model is split.

## Compatibility work before JavaScript

The next engine work should improve deterministic parsing/layout behavior before introducing a scripting runtime:

```text
CSS tokenizer / declaration parser
              |
              v
Unicode line breaking + shaping
              |
              v
Inline formatting context
              |
              v
Box model / selectors / media queries
              |
              v
Same-Origin foundation + forms/storage
              |
              v
Fuzzing + WPT subset
              |
              v
Cherry JavaScript frontend / VM
```

This sequencing keeps the future JavaScript runtime from depending on unstable DOM/style/layout behavior.

## Long-term process model

```text
+---------------- Browser Process ----------------+
| tabs | history | permissions | downloads | UI  |
+----------------------+---------------------------+
                       |
                       | IPC
                       v
+--------------- Renderer Process ----------------+
| DOM | CSS | JS VM | Style | Layout | Paint      |
+----------------------+---------------------------+
                       |
                       | display lists / surfaces
                       v
+-------------- Compositor Process ---------------+
| GPU resources | raster | layers | compositing   |
+--------------------------------------------------+
                       ^
                       |
+----------------------+---------------------------+
| isolated network service | cache | cookie jar   |
+--------------------------------------------------+
```

Network and storage services become isolated processes after the single-process engine is functionally mature enough to make the IPC boundary worth the complexity.

## Dependency policy

Top-level infrastructure dependencies are pinned and `Cargo.lock` is committed because CherryBrowser is an application, not a reusable Rust library. CI builds and tests with the lockfile so an unrelated upstream patch release cannot silently change the validated engine build.
