# CherryBrowser

CherryBrowser is an **independent browser engine written in Rust**.

The project does **not** embed Chromium/Blink, WebKit, Firefox/Gecko, CEF, Electron, or a webview. The HTML parser, DOM, CSS cascade, layout model, and page renderer in this repository are CherryBrowser code.

General-purpose Rust libraries are used for operating-system UI and HTTP/TLS plumbing. Those are infrastructure dependencies, not browser engines.

## Milestone 0.1

The first milestone intentionally starts small and executable instead of claiming fake compatibility with the modern web.

Implemented:

- Native desktop browser shell
- URL bar
- Back / Forward / Reload
- HTTP and HTTPS navigation
- Redirect handling
- Independent DOM tree
- Independent HTML tokenizer/parser
- Basic entity decoding
- Raw `<style>` and `<script>` handling
- Independent CSS parser
- Tag, class, ID and descendant selector matching
- Basic cascade and inline styles
- Block and inline flow layout
- Text wrapping
- Basic colors, margins, padding, font sizes and backgrounds
- Headings, paragraphs, links, lists and image placeholders
- Clickable links
- Background network worker so navigation does not freeze the window

Not implemented yet:

- JavaScript runtime
- External stylesheets
- Real image decoding/fetching
- HTML5 error-recovery compatibility
- CSS Flexbox/Grid
- Forms
- Cookies/storage
- Multi-process sandbox
- GPU page renderer independent from the temporary native UI shell
- HTTP cache
- DevTools
- Accessibility tree

Modern sites that require JavaScript will therefore not work yet. That is expected. A browser engine is not conjured into standards compliance by changing a logo and adding 800 MB of someone else's engine.

## Architecture

```text
URL
 |
 v
Network (HTTP/HTTPS)
 |
 v
Cherry HTML Parser
 |
 v
Cherry DOM
 |
 +----> Cherry CSS Parser / Cascade
 |                |
 v                v
        Cherry Layout Engine
                |
                v
           Display List
                |
                v
         Native Renderer
```

Source layout:

```text
src/
├── app.rs       Browser shell, navigation and history
├── net.rs       HTTP/HTTPS transport boundary
├── dom.rs       DOM tree
├── html.rs      HTML tokenizer/parser
├── css.rs       CSS parser, selectors and cascade
├── layout.rs    Block/inline layout and display list
├── renderer.rs  Native painting and link hit testing
├── lib.rs
└── main.rs
```

## Build

Install stable Rust, then:

```bash
cargo run --release
```

On Linux you need the normal X11/Wayland development packages required by the native window stack.

## Engineering rule

CherryBrowser may use libraries for generic primitives such as TLS, sockets, graphics APIs, fonts, compression, cryptography and OS integration.

It must not quietly replace its browser engine with Chromium/Blink, WebKit, Gecko, CEF, Electron, a system WebView, or another browser renderer.

## Next engine milestones

1. Fetch and cascade external CSS
2. Fetch/decode raster images
3. Correct inline formatting contexts and box model
4. CSS specificity, inheritance and media queries
5. Forms and input events
6. Cookies, cache and origin storage
7. JavaScript parser/bytecode VM
8. DOM bindings and event loop
9. Dedicated GPU compositor
10. Site isolation and sandboxing
11. HTTP/2 and HTTP/3 tuning
12. Compatibility test harness and performance benchmarks

No license has been selected yet.
