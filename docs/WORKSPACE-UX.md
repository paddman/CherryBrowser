# Usable workspace UX

This change builds on the existing native Rust/egui shell. It does not replace or broaden the Cherry web engine. New Tab, Research, Bookmarks, History and Settings are first-party workspace sections, **not independent browser tabs**.

## Working interactions

- The omnibox and New Tab search classify submitted URLs versus search terms. Search uses DuckDuckGo HTML by default, or Bing when selected in Settings. Typing does not make requests. Explicit unsupported schemes and credential-bearing URLs are rejected.
- Starter links and bookmarks route through the existing Cherry document loader, not the operating system's browser.
- Bookmarks support create, edit, remove and filtering, with 128 entries maximum and 4096-byte URL limits. Ctrl/Cmd+D bookmarks the last completed page, never an in-flight target.
- Research has editable Unicode notes, source URL insertion, return-to-page, source bookmarking and explicit clipboard copy. Notes have a 16,000-character limit.
- Save workspace persists bookmarks, notes, search provider and appearance preferences in a versioned local text file. Saving is explicit; a visible dirty banner warns that closing without saving discards edits. Data is **not encrypted** and should not contain secrets.
- History displays actual completed navigation, including successful Back/Forward visits. It is bounded to 200 entries and is never written to the workspace file. Clearing resets Back/Forward. An in-flight traversal is cancelled before its history index is removed.
- Ctrl/Cmd+K opens a filterable command palette with Up/Down, Enter and Escape. Ctrl/Cmd+L focuses and selects the omnibox. Ctrl/Cmd+H/B opens History/Bookmarks; Alt+Left/Right navigates; Ctrl/Cmd+R or F5 reloads; Escape stops a pending request.
- Stop cancels the current worker's token and restores the last completed page or workspace. Retry retains the failed request's history mode. Failed loads are shown above, not hidden beneath, stale page content.

## Appearance

Deep navy surfaces, electric blue, softer violet/cyan accents, stronger text hierarchy, less border noise and more breathing room. The short-blue-haired Cherry companion is kept as native vector art. No fruit, shields, keys or padlocks are used.

The left rail appears at 1000 content pixels. The optional companion appears at 1320 content pixels. Below those breakpoints, navigation wraps above the single content column; the renderer is not given a fake fixed-width three-column canvas. Focus mode removes the hero and companion. These are shell breakpoints, not new CSS/media-query support in web pages.

No dummy AI input, fake security score, fake tracker/secure-DNS status, sample user history, simulated tab counts or decorative dead buttons remain in the live first-party screens. AI and unimplemented engine features are labelled unavailable. `design_system.rs` remains a developer design reference and is no longer a home dashboard widget.

## Storage

- Windows: `%LOCALAPPDATA%/CherryBrowser/workspace.txt`
- macOS: `$HOME/Library/Application Support/CherryBrowser/workspace.txt`
- Linux: `$XDG_DATA_HOME/CherryBrowser/workspace.txt`, falling back to `$HOME/.local/share/CherryBrowser/workspace.txt`

A 512 KiB read limit, version/record validation, URL validation and field limits apply. Saving writes a same-directory temporary file, flushes it, and renames it over the destination. Unix temporary files are mode 0600. An unreadable or invalid original file is left untouched and persistent saving is disabled for that run rather than silently overwriting it. No new dependencies, browser-engine substitutions, account connections or network-enabled assistant services are introduced.

## Review and test checklist

Run the existing checks with the committed lockfile:

```sh
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

Automated unit coverage includes query encoding (Thai and URL delimiters), URL/host-port classification, unsafe schemes/credentials, empty/oversized input, persistence round trips, malformed records, bookmark deduplication/editing/limits, atomic replacement, responsive breakpoint decisions, and command availability. These are not native screenshot or end-to-end GUI tests.

Manual acceptance before merge:

1. Check 640, 1024 and 1440-pixel windows at 100% and 150% OS scaling. Confirm no clipped controls, readable labels and optional rails. Also inspect a narrow window, long URLs and Thai text.
2. Enter `example.com`; enter a Thai search; click a starter link and a saved bookmark. Confirm all use Cherry's loader. Search pages may still render partially because the engine does not execute JavaScript.
3. Create/edit/delete/filter a bookmark. Type notes, insert a loaded page's URL, save, close and restart; verify exact recovery. Unsaved edits must remain visibly identified. The workspace file must contain no browsing history.
4. Navigate A → B, go Back, then visit C. Verify Forward truncation and the independent session history view. Clear history and verify disabled Back/Forward without discarding the current page.
5. Submit a slow URL, supersede it, stop it, and open Home while loading. A cancelled worker must not replace the active surface later.
6. Load a good page followed by a failing address. Verify visible error/Retry and that returning to the prior page restores its URL/status. No failed URL should appear in history or be bookmarked as if it loaded.
7. Open the palette from the omnibox, filter, use arrows/Enter/Escape, then use Ctrl/Cmd+L. Enter must execute only one command, never a background URL submission.
8. Verify Settings focus mode and companion toggles change the UI. Verify AI/security disclosures are factual. No GPU/LLM actions or packet enforcement is involved.

Known scope boundaries: no real multi-tab engine, no AI backend, no new origin security model, no browser sandbox, no CSS-layout milestone advancement, no download manager, and no encrypted/synchronised workspace storage. The engine remains milestone 0.3.4; package/release versioning is deliberately not bumped by this UX branch.
