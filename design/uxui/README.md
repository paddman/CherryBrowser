# CherryBrowser Master UX/UI

This folder is the implementation reference for the merged CherryBrowser UX/UI direction.

## Direction

- Electric blue + violet + white cyber-tech palette
- Dense but structured glass-style surfaces
- One dominant browser canvas instead of disconnected dashboard cards
- Left navigation for browser/workspace context
- Center content for New Tab, Research, and page rendering
- Right Cherry AI assistant rail
- Bottom context dock for research/session/design-system tools
- City/network motifs are decorative layers, not replacements for browser content
- Cherry is a short-blue-haired assistant character; do not use cherry-fruit imagery

## Implemented native screens

The design has now been split into first-party Rust/egui modules rather than living as one giant `app.rs` mockup:

```text
src/ui/
├── mod.rs             Shell page routing
├── theme.rs           Shared visual tokens / frames
├── home.rs            Main responsive three-column shell
├── sidebar.rs         Navigation + workspace rail
├── assistant.rs       Cherry AI assistant rail
├── workspace.rs       Quick links, feature cards, context dock
├── research.rs        Research workspace screen
├── settings.rs        Settings / appearance / privacy screen
└── design_system.rs   Theme swatches and reusable component samples
```

The native shell currently exposes three switchable first-party views: **New Tab**, **Research Workspace**, and **Settings / Design System**. The actual web-page renderer remains the independent Cherry engine and is not replaced by these surfaces.

## Master information architecture

```text
Window chrome
├─ Tabs
├─ Back / Forward / Reload
├─ Address / Search
└─ Profile / Window actions

Application shell
├─ Left rail
│  ├─ New Tab
│  ├─ AI Assistant
│  ├─ Research Workspace
│  ├─ Bookmarks
│  ├─ History
│  ├─ Downloads
│  ├─ Security Insights
│  └─ Themes & Settings
│
├─ Main canvas
│  ├─ Home hero + private search
│  ├─ Quick links
│  ├─ Product / research cards
│  ├─ Research workspace
│  ├─ Settings / design system
│  └─ Actual page renderer
│
├─ Cherry assistant rail
│  ├─ Chat
│  ├─ Search
│  ├─ Tools
│  ├─ Summarize
│  ├─ Translate / Rewrite
│  └─ Open in workspace
│
└─ Context dock
   ├─ Research Workspace
   ├─ Saved Notes & Snippets
   ├─ Security & Privacy Insights
   ├─ Themes / Layout / Design System
   └─ Recent Sessions
```

## Design tokens

| Token | Value |
|---|---|
| Background | `#040915` |
| Surface | `#081125` |
| Surface elevated | `#0D1831` |
| Primary | `#3D96FF` |
| Violet | `#8E57FF` |
| Cyan | `#40E2FF` |
| Text | `#E1EEFF` |
| Muted | `#97A8C9` |
| Positive | `#36D399` |
| Corner radius | `8-14px` visual equivalent |

## Implemented interaction path

1. `CherryApp` owns a `ShellPage` state while the first-party home shell is visible.
2. Top tabs and left navigation can switch between New Tab, Research, and Settings without invoking the web loader.
3. The New Tab URL field still hands navigation back to the existing Cherry network / document pipeline.
4. Pressing Home resets the first-party shell to New Tab.
5. Real loaded pages continue through the existing DOM, CSS, layout, display-list, and renderer path.

## Next implementation order

1. Persist workspace and tab state instead of demo rows
2. Route quick links and search suggestions into real navigation
3. Connect Cherry assistant actions to a local/provider abstraction
4. Attach notes/snippets to real page/session IDs
5. Persist themes and settings
6. Add real browsing security telemetry to Security Insights
7. Optional character/background raster assets after shell behavior is stable

The UI must remain separate from engine semantics: CherryBrowser still owns navigation, DOM/CSS, layout, resource loading, and page rendering.
