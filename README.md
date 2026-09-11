# CherryBrowser

CherryBrowser is an **independent browser engine written in Rust**.

The project does **not** embed Chromium/Blink, WebKit, Firefox/Gecko, CEF, Electron, or a system WebView. The HTML parser, DOM, CSS parser/cascade, resource orchestration, layout model, display list, hit testing, and page-rendering semantics in this repository are CherryBrowser code.

General-purpose Rust libraries are used only for infrastructure primitives such as native OS UI, HTTP/TLS transport, compression, and image codecs. They are not browser engines.

## Milestone 0.2

CherryBrowser now has an end-to-end native browsing pipeline with external stylesheets and raster images.

Implemented:

- Native desktop browser shell
- URL bar
- Back / Forward / Reload
- HTTP and HTTPS navigation
- Redirect handling
- Shared HTTP client and connection pooling across document/subresource requests
- Independent DOM tree
- Independent HTML tokenizer/parser
- DOM parsed once and reused through style/layout
- Basic entity decoding
- Raw `<style>` and `<script>` handling
- `<base href>` support for both resource resolution and clicked links
- External `<link rel="stylesheet">` loading
- Parallel stylesheet fetches
- Stylesheet deduplication plus count/size safety limits
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
- Parallel `<img src>` fetch/decode pipeline
- PNG, JPEG, and WebP decoding
- Intrinsic image sizing and aspect-ratio preservation when possible
- Image display-list items and per-page texture cache
- Clickable links
- Background document/resource worker so navigation does not freeze the window
- Pinned top-level dependency versions and committed `Cargo.lock`
- Rust formatting, compile, and unit-test checks in GitHub Actions

### Resource safety budgets

Current milestone limits are intentionally conservative while the engine matures:

- Document body: 8 MiB
- External stylesheet: 2 MiB each
- External stylesheets: maximum 24 per document
- Image download: 12 MiB each
- Images: maximum 32 per document
- Decoded image dimensions: maximum 8192 × 8192
- Decoder allocation budget: 128 MiB per image (best-effort where the codec cannot enforce it strictly)

## Not implemented yet

- JavaScript runtime
- HTML5-complete error recovery
- Full CSS origins/layers and the complete modern selector grammar
- CSS Flexbox/Grid
- Full media-query evaluation
- `srcset`, `<picture>`, responsive image selection, AVIF, GIF animation, SVG rendering
- Forms and input event semantics
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
 |
 v
Cherry Document Loader
 |              \
 |               +--> parallel CSS fetches
 |               +--> parallel image fetch/decode
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
├── app.rs         Browser shell, navigation and history
├── loader.rs      Document/subresource orchestration
├── net.rs         HTTP/HTTPS transport and resource budgets
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

1. Correct inline formatting contexts and more of the CSS box model
2. Attribute selectors, pseudo classes, and media-query evaluation
3. Forms and input events
4. Cookie jar, HTTP cache, and origin storage
5. Bounded resource worker pool and smarter image/CSS caching
6. JavaScript tokenizer/parser and AST
7. Bytecode VM and garbage collector
8. DOM bindings, event loop, timers, and Fetch APIs
9. Dedicated GPU compositor
10. Multi-process renderer sandbox and site isolation
11. HTTP/2 and HTTP/3 tuning
