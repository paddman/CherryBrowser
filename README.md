# CherryBrowser

CherryBrowser is an **independent browser engine written in Rust**.

The project does **not** embed Chromium/Blink, WebKit, Firefox/Gecko, CEF, Electron, or a system WebView. The HTML parser, DOM, CSS parser/cascade, resource orchestration, layout model, display list, hit testing, navigation policy, and page-rendering semantics in this repository are CherryBrowser code.

General-purpose Rust libraries are used only for infrastructure primitives such as native OS UI, HTTP/TLS transport, compression, and image codecs. They are not browser engines.

## Milestone 0.3.1

CherryBrowser has an end-to-end native browsing pipeline and is now hardening compatibility, cancellation, resource safety, and legacy text decoding before larger web-platform features.

Implemented:

- Native desktop browser shell
- URL bar
- Back / Forward / Reload
- HTTP and HTTPS navigation
- Redirect handling
- Shared HTTP client and connection pooling across document/subresource requests
- Streaming document/CSS/image body limits
- Cooperative cancellation for superseded navigations
- Cancellation checks while streaming response bodies and between subresource stages
- Independent DOM tree
- Independent HTML tokenizer/parser
- DOM parsed once and reused through style/layout
- Basic entity decoding
- Raw `<style>` and `<script>` handling
- `<base href>` support for both resource resolution and clicked links
- Improved malformed end-tag handling
- Correct preservation of `/` in unquoted HTML attribute values
- First duplicate HTML attribute wins
- External `<link rel="stylesheet">` loading
- Bounded stylesheet/image fetch concurrency
- Stylesheet deduplication plus count/size safety limits
- Repeated image URL fetch/decode deduplication
- Embedded and external CSS merged in DOM order
- Independent CSS parser
- Tag, class, ID and descendant selector matching
- Unsupported combinators/pseudo/attribute selectors rejected instead of silently broadening matches
- `!important`, inline style, specificity, source-order, and declaration-order priority handling
- Safe skipping of unsupported at-rules
- Block and inline flow layout
- Text wrapping
- Basic colors, margins, padding, widths, heights, font sizes, and backgrounds
- Headings, paragraphs, links, lists, and image elements
- PNG, JPEG, and WebP decoding
- Intrinsic image sizing and aspect-ratio preservation when possible
- RGBA allocation safety checks before final image conversion
- Image display-list items and per-page texture cache
- Clickable links
- Nested background/image placeholder stacking fix
- Background document/resource worker so navigation does not freeze the window
- Back/Forward history position committed only after successful traversal
- Page title surfaced in the browser status UI
- Web text decoding from UTF BOMs, HTTP `charset`, and HTML `<meta charset>`
- Initial legacy encoding support for Windows-1252 and Windows-874/TIS-620, including Thai legacy pages
- Pinned top-level dependency versions and committed `Cargo.lock`
- Rust formatting, compile, Clippy-with-warnings-denied, and unit-test checks in GitHub Actions

### Resource safety budgets

Current milestone limits are intentionally conservative while the engine matures:

- Document body: 8 MiB
- External stylesheet: 2 MiB each
- External stylesheets: maximum 24 per document
- Image download: 12 MiB each
- Images: maximum 32 per document
- Concurrent CSS/image workers: maximum 6 per batch
- Decoded image dimensions: maximum 8192 × 8192
- Decoder/RGBA allocation budget: 128 MiB per image
- Redirects: maximum 10
- Network timeout: 25 seconds
- Connect timeout: 10 seconds

## Not implemented yet

- JavaScript runtime
- HTML5-complete error recovery
- Full CSS tokenizer/grammar, origins/layers, and modern selector grammar
- Correct complete inline formatting context and text shaping
- Unicode/Thai line-breaking engine independent from whitespace splitting
- CSS Flexbox/Grid
- Full media-query evaluation
- `srcset`, `<picture>`, responsive image selection, AVIF, GIF animation, SVG rendering
- Forms and input event semantics
- Same-Origin Policy/CORS/CSP security model
- Cookies and origin storage
- HTTP cache
- WebSocket / Fetch web APIs
- Multi-process renderer sandbox
- Site isolation
- Dedicated Cherry GPU compositor independent from the temporary native UI shell
- DevTools
- Accessibility tree

Modern JavaScript-heavy sites will still fail or render partially. That is expected. CherryBrowser is building the engine instead of quietly shipping somebody else's engine under a different toolbar.

## Architecture

```text
URL
 |
 v
Cherry Network Layer
shared HTTP/TLS connection pool
streaming safety limits + cancellation
 |
 v
Cherry Text Decoder
BOM / HTTP charset / HTML meta charset
 |
 v
Cherry Document Loader
 |              \
 |               +--> bounded parallel CSS fetches
 |               +--> bounded/deduplicated image fetch/decode
 v
Cherry HTML Parser
 |
 v
Cherry DOM
 |       \
 |        +--> Cherry CSS parser / cascade
 |                    |
 +--------------------+
          |
          v
 Cherry Layout Engine
          |
          v
     Display List
      |       |
      |       +--> image paint items / texture cache
      v
  Native Renderer
```

Source layout:

```text
src/
├── app.rs         Browser shell, navigation, cancellation and history
├── cancel.rs      Cooperative cancellation token
├── loader.rs      Document/subresource orchestration
├── net.rs         HTTP/HTTPS transport and streaming resource budgets
├── text.rs        Web text charset/BOM/meta decoding
├── dom.rs         DOM tree
├── html.rs        HTML tokenizer/parser
├── css.rs         CSS parser, selectors and cascade
├── image_data.rs  Bounded PNG/JPEG/WebP decoding
├── layout.rs      Block/inline layout and display list
├── renderer.rs    Native painting, image textures and link hit testing
├── lib.rs
└── main.rs
```

## Build

Install stable Rust, then:

```bash
cargo run --release --locked
```

On Linux you need the normal X11/Wayland development packages required by the native window stack.

## Engineering rule

CherryBrowser may use libraries for generic primitives such as TLS, sockets, graphics APIs, fonts, image codecs, compression, cryptography, and OS integration.

It must not replace its browser engine with Chromium/Blink, WebKit, Gecko, CEF, Electron, a system WebView, or another browser renderer.

## Next engine milestones

1. Real CSS tokenization/declaration parsing before adding larger CSS layout features
2. Unicode line breaking, Thai-aware wrapping, grapheme handling, and real glyph measurement/shaping
3. Correct inline formatting contexts and more of the CSS box model
4. Attribute selectors, pseudo classes, media-query evaluation, then Flexbox/Grid
5. Same-Origin Policy foundation, forms/input events, cookie jar, HTTP cache, and origin storage
6. Parser fuzzing, rendering regression tests, and a Web Platform Tests subset harness
7. JavaScript tokenizer/parser and AST
8. Bytecode VM and garbage collector
9. DOM bindings, event loop, timers, promises, and Fetch APIs
10. Dedicated GPU compositor
11. Multi-process renderer sandbox and site isolation
12. HTTP/2 and HTTP/3 tuning
