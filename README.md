# CherryBrowser

CherryBrowser is an **independent browser engine written in Rust**.

The project does **not** embed Chromium/Blink, WebKit, Firefox/Gecko, CEF, Electron, or a webview. The HTML parser, DOM, CSS cascade, layout model, resource orchestration, and page renderer in this repository are CherryBrowser code.

General-purpose Rust libraries are used only for infrastructure such as native OS UI and HTTP/TLS transport. They are not browser engines.

## Milestone 0.2

CherryBrowser now has an end-to-end native browsing pipeline and the beginning of a real subresource loader.

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
- **External `<link rel="stylesheet">` loading**
- **Parallel stylesheet fetches**
- **`<base href>` support for stylesheet URL resolution**
- **Stylesheet count and size safety limits**
- Embedded and external CSS merged in DOM order
- Independent CSS parser
- Tag, class, ID and descendant selector matching
- Basic cascade and inline styles
- Block and inline flow layout
- Text wrapping
- Basic colors, margins, padding, font sizes and backgrounds
- Headings, paragraphs, links, lists and image placeholders
- Clickable links
- Background document/resource worker so navigation does not freeze the window
- Rust formatting, compile and unit-test checks in GitHub Actions

Not implemented yet:

- JavaScript runtime
- Real image decoding/fetching
- HTML5 error-recovery compatibility
- Full CSS cascade semantics (`!important`, origins, layers)
- CSS Flexbox/Grid
- Media query evaluation beyond basic stylesheet media filtering
- Forms
- Cookies/storage
- Multi-process sandbox
- Dedicated GPU page compositor independent from the temporary native UI shell
- HTTP cache
- DevTools
- Accessibility tree

Modern JavaScript-heavy sites will still fail or render partially. That is expected. The project is building the engine instead of quietly shipping somebody else's engine under a different toolbar.

## Architecture

```text
URL
 |
 v
Network (HTTP/HTTPS)
 |
 v
Cherry Document Loader
 |                \
 |                 +--> External CSS fetch workers
 v
Cherry HTML Parser
 |
 v
Cherry DOM -----> CSS source ordering / cascade
 |                         |
 v                         v
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
├── loader.rs    Document/subresource orchestration
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

CherryBrowser may use libraries for generic primitives such as TLS, sockets, graphics APIs, fonts, image codecs, compression, cryptography and OS integration.

It must not replace its browser engine with Chromium/Blink, WebKit, Gecko, CEF, Electron, a system WebView, or another browser renderer.

## Next engine milestones

1. Correct `!important` and stronger CSS cascade semantics
2. Fetch/decode raster images with explicit resource budgets
3. Correct inline formatting contexts and box model
4. CSS attribute selectors, pseudo classes and media queries
5. Forms and input events
6. Cookies, cache and origin storage
7. JavaScript tokenizer/parser
8. Bytecode VM, garbage collector and DOM bindings
9. Event loop, timers and fetch APIs
10. Dedicated GPU compositor
11. Site isolation and sandboxing
12. HTTP/2 and HTTP/3 tuning
