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
│  ├─ Workspaces
│  ├─ Bookmarks
│  ├─ History
│  ├─ Downloads
│  ├─ Security Insights
│  └─ Extensions
│
├─ Main canvas
│  ├─ Home hero + private search
│  ├─ Quick links
│  ├─ Product / research cards
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
| Corner radius | `10-14px` visual equivalent |

## Implementation order

1. Browser chrome and responsive three-column shell
2. New Tab / Home layout
3. Workspace switcher and contextual bottom dock
4. Cherry assistant rail
5. Research workspace states
6. Settings / themes / design-system views
7. Optional character/background raster assets after shell behavior is stable

The UI must remain separate from engine semantics: CherryBrowser still owns navigation, DOM/CSS, layout, resource loading, and page rendering.