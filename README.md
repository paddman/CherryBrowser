# CherryBrowser

CherryBrowser is an **independent browser engine written in Rust**.

The project does **not** embed Chromium/Blink, WebKit, Firefox/Gecko, CEF, Electron, or a system WebView. The HTML parser, DOM, CSS parser/cascade, resource orchestration, layout model, display list, hit testing, navigation policy, and page-rendering semantics in this repository are CherryBrowser code.

General-purpose Rust libraries are used only for infrastructure primitives such as native OS UI, HTTP/TLS transport, compression, font rasterization/measurement, and image codecs. They are not browser engines.

## Milestone 0.3.4

CherryBrowser has an end-to-end native browsing pipeline and now uses renderer-backed font metrics plus bounded operating-system font fallbacks for multilingual text. Cherry still owns text break policy and layout geometry; the native font stack supplies the primitive glyph measurement/rasterization used by the temporary UI shell.

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
- Independent CSS parser and cascade
- Syntax-aware CSS comment scanning that preserves comment markers inside quoted strings
- Top-level CSS declaration splitting that respects strings, escapes, functions, brackets, and nested blocks
- CSS values containing data URLs, colons, or semicolons no longer split declarations incorrectly
- Top-level `!important` parsing without treating quoted/function content as a priority marker
- Custom property names preserve case and custom properties inherit through the current style map
- Tag, class, ID and descendant selector matching
- Unsupported combinators/pseudo/attribute selectors rejected instead of silently broadening matches
- Invalid selector components and selector-list members reject the rule instead of partially applying it
- `!important`, inline style, specificity, source-order, and declaration-order priority handling
- Safe skipping of unsupported at-rules
- Block and inline flow layout
- Unicode-aware text break units instead of whitespace-only wrapping
- Thai fallback break opportunities for long text without ASCII spaces
- Thai combining marks and leading-vowel clusters kept together during line breaking
- CJK break opportunities between ideographic/kana/hangul clusters
- Combining marks, variation selectors, emoji modifiers, regional-indicator pairs, and ZWJ emoji sequences kept in visual clusters
- Script/cluster-aware width estimator retained for headless and pre-native-metric fallback
- Native UI font measurement used for page layout after the first UI pass
- Automatic relayout after the native text measurer becomes available
- Bounded discovery of relevant system Thai/CJK/emoji fonts on Windows, Linux and macOS
- System fallback fonts inserted at the lowest family priority so normal Latin rendering is not replaced unnecessarily
- No font binaries bundled in this repository
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
- System font candidates scanned: maximum 512 files
- System fallback fonts installed: maximum 8
- System fallback font size: maximum 32 MiB each
- Total installed system fallback font bytes: maximum 64 MiB
- Font directory recursion depth: maximum 4
- Redirects: maximum 10
- Network timeout: 25 seconds
- Connect timeout: 10 seconds

## Not implemented yet

- JavaScript runtime
- HTML5-complete error recovery
- Full standards-complete CSS tokenizer/grammar, origins/layers, and modern selector grammar
- CSS escape decoding and the complete identifier grammar
- `var()` substitution and full custom-property computed-value semantics
- Full Unicode Line Breaking Algorithm coverage and language-dictionary segmentation
- Browser-owned font selection/shaping independent from the temporary native UI font stack
- Bidirectional text layout and complete Arabic/Indic shaping behavior
- Correct complete inline formatting context and baseline calculation
- Web-font loading through `@font-face`
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

System fallback availability is platform-dependent. If a suitable Thai/CJK/emoji system font is not installed, CherryBrowser keeps using the bounded fallback estimator and the temporary native painter may still lack glyph coverage. Modern JavaScript-heavy sites will still fail or render partially. That is expected.

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
 |        +--> Cherry CSS syntax scanner / parser / cascade
 |                    |
 +--------------------+
          |
          v
 Cherry Text Layout Helpers
 cluster-safe break units
 Thai/CJK fallback wrapping
          |
          +-------------------------+
          |                         |
          v                         v
 native font measurer       bounded fallback estimator
 system font fallback
          |                         |
          +------------+------------+
                       |
                       v
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
├── app.rs          Browser shell, navigation, cancellation and native metric hookup
├── cancel.rs       Cooperative cancellation token
├── loader.rs       Document/subresource orchestration
├── net.rs          HTTP/HTTPS transport and streaming resource budgets
├── text.rs         Web text charset/BOM/meta decoding
├── text_layout.rs  Unicode cluster/break helpers plus native/fallback width metrics
├── font_support.rs Bounded system font fallback discovery and installation
├── dom.rs          DOM tree
├── html.rs         HTML tokenizer/parser
├── css.rs          CSS rules, selectors, declarations and cascade
├── css_syntax.rs   CSS syntax-aware scanning and top-level splitting
├── image_data.rs   Bounded PNG/JPEG/WebP decoding
├── layout.rs       Block/inline layout and display list
├── renderer.rs     Native painting, image textures and link hit testing
├── lib.rs
└── main.rs
```

## Build

Install stable Rust, then:

```bash
cargo run --release --locked
```

On Linux you need the normal X11/Wayland development packages required by the native window stack. Multilingual rendering also depends on suitable fonts already installed by the operating system; CherryBrowser does not ship font binaries.

## Engineering rule

CherryBrowser may use libraries for generic primitives such as TLS, sockets, graphics APIs, fonts, text shaping primitives, image codecs, compression, cryptography, and OS integration.

It must not replace its browser engine with Chromium/Blink, WebKit, Gecko, CEF, Electron, a system WebView, or another browser renderer. Font measurement and rasterization are infrastructure primitives; Cherry still owns DOM/style semantics, break decisions, line construction, layout geometry, and display-list generation.

## Next engine milestones

1. Correct inline formatting contexts, baseline calculation, and stronger bidi/script shaping behavior
2. More of the CSS box model, attribute selectors, pseudo classes, and media-query evaluation
3. Flexbox/Grid after box/inline semantics are stable
4. Same-Origin Policy foundation, forms/input events, cookie jar, HTTP cache, and origin storage
5. Parser fuzzing, rendering regression tests, and a Web Platform Tests subset harness
6. JavaScript tokenizer/parser and AST
7. Bytecode VM and garbage collector
8. DOM bindings, event loop, timers, promises, and Fetch APIs
9. Dedicated GPU compositor
10. Multi-process renderer sandbox and site isolation
11. HTTP/2 and HTTP/3 tuning
