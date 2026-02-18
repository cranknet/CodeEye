# CodeEye — Product Requirements Document

> *"The fastest way to report UI bugs to AI"*

**Version:** 2.0
**Date:** 2026-02-18
**Status:** Architecture finalized, pre-development

---

## 1. Vision

Developers spot UI issues constantly but communicating them is slow. You screenshot, switch to chat, describe the problem in words, hope the AI understands which pixel you mean. By the time you've explained it, you could have fixed it yourself.

CodeEye eliminates that friction. Global hotkey, drag to capture, draw on the screenshot, press Enter. The AI gets a structured prompt with annotated image, pixel coordinates, git context, and your comments — and starts fixing immediately.

**Core principle:** Every feature must reduce time-to-feedback. If it adds a step, it doesn't ship.

---

## 2. Audience

**Primary:** Developers using AI coding assistants (Claude Code, Codex, Gemini CLI, OpenCode, Aider, Cline, Continue) for frontend work.

**Secondary:** Developers who file visual bugs to GitHub Issues or Linear, with or without AI.

**Tertiary:** Designers reviewing implementation against mockups.

---

## 3. Platform & Tech Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Runtime | Tauri v2 | ~8MB binary, native APIs, cross-platform |
| Frontend | Svelte 5 + Vite + Tailwind CSS | Tiny bundle, reactive canvas state, Tauri's first-class framework |
| Backend | Rust (Tauri core) | Screen capture, global shortcuts, MCP server, file I/O |
| MCP Protocol | Embedded stdio server | No separate process — runs inside Tauri |
| Storage | File-based (`~/.codeeye/`) | Sessions, settings, project index |
| Canvas | Canvas 2D API | Single element, requestAnimationFrame render loop |
| State | Svelte 5 Runes | `$state`, `$derived`, `$effect` everywhere; pure TS for geometry utils |
| Linting | Ultracite (Svelte/TS), cargo fmt + clippy (Rust) | Local pre-commit, not CI |
| Testing | Vitest (TS), cargo test (Rust), Tauri WebDriver (E2E) | Run locally before commit |
| i18n | Paraglide JS (or svelte-i18n) | i18n-ready from day one, English only at launch |

**Build targets:**

| Platform | Formats | Est. Size |
|----------|---------|-----------|
| macOS (Intel + ARM) | `.dmg` | ~8MB |
| Windows 10/11 | `.msi` + `.exe` | ~6MB |
| Debian / Ubuntu | `.deb` | ~7MB |
| Fedora | `.rpm` | ~7MB |
| Linux generic | `.AppImage` | ~9MB |

---

## 4. Feature Tiers

### Tier 1 — Core (MVP Launch)

#### 4.1 Screen Capture

**Primary flow — Global hotkey + region select:**
1. Press `Cmd+Shift+E` (Mac) / `Ctrl+Shift+E` (Win/Linux) from any app
2. Detect which monitor has the cursor
3. Dim overlay on that monitor only (Tauri transparent fullscreen window)
4. Drag crosshair to select region
5. **Keyboard resize phase:** Arrow keys nudge edges (1px, `Shift+Arrow` 10px), `Alt+Arrow` resizes opposite edge, live dimension display
6. `Enter` confirms selection, `Escape` cancels
7. Capture region at native resolution, run compression pipeline
8. CodeEye editor opens with capture loaded, ready to annotate
9. Time from hotkey to first annotation: **under 2 seconds**

**Multi-monitor awareness:**
- Detect all connected displays on hotkey press
- Dim only the display where the cursor is
- If region spans two monitors, capture both and stitch
- Store display metadata (resolution, scaling factor) per capture

**Secondary flow — Manual upload:**
- Drag & drop image onto CodeEye window
- Paste from clipboard (`Cmd+V` / `Ctrl+V`)
- Click to browse file system
- All inbound paths converge: image bytes → compression pipeline → canvas

**System tray:**
- CodeEye lives in the system tray, ~15MB RAM idle
- Single instance enforced via Tauri `single-instance` plugin
- Tray menu: New Capture, Recent Sessions (last 5), Open CodeEye, Quit
- Badge indicator when unresolved MCP feedback exists

**macOS permissions:**
- On first capture attempt, check for screen recording permission
- If missing: guide user to System Settings → Privacy & Security → Screen Recording
- Verify permission granted before proceeding

#### 4.2 Annotation Tools

| Tool | Shortcut | Behavior |
|------|----------|----------|
| Select / Move | `V` | Click to select, drag to move, corner handles to resize |
| Circle | `C` | Drag bounding box → inscribed ellipse |
| Square | `S` | Drag bounding box → rounded rectangle |
| Arrow | `A` | Click start → drag to end, arrowhead at tip |
| Freehand | `F` | Click-drag draws smooth path |
| Text | `T` | Click to place, inline editable label on canvas |

**Common behaviors:**
- Every annotation gets a numbered marker (#1, #2, ...) and optional comment via popup
- Default color: orange (`#f97316`). Optional color coding: red (critical), yellow (minor), blue (suggestion)
- **Annotation outline/shadow:** All annotations render with a subtle outline/drop shadow for visibility against any screenshot background
- `Delete` removes selected annotation
- `Ctrl+Z` / `Cmd+Z` undo, `Ctrl+Shift+Z` / `Cmd+Shift+Z` redo (state snapshot model)
- `Shift+Click` for multi-select, batch delete

**Quick labels / presets:**
- Built-in presets: `spacing`, `alignment`, `color`, `font`, `overflow`, `responsive`, `z-index`, `missing element`
- User-defined custom presets
- Click preset → click on canvas → annotation placed with label pre-filled
- Each preset has a default severity (overridable): e.g., `overflow` → critical, `spacing` → minor

#### 4.3 Canvas Interaction

- **Zoom:** Scroll wheel zooms toward cursor. `Ctrl+0` / `Cmd+0` resets to fit. Zoom level indicator in corner.
- **Pan:** Middle-click drag, or `Space` + drag. Auto-pans when drawing near canvas edges.
- **Move & Resize:** Select tool (`V`) — click annotation to select, drag to reposition, corner handles to scale. Aspect ratio lock with `Shift`.

**Rendering pipeline (Canvas 2D):**
```
Screenshot (base image)
  → Transform (zoom/pan matrix)
    → Annotation layer (shapes, arrows, text, with outline/shadow)
      → Selection handles (resize grips, bounding boxes)
        → Cursor overlay (crosshair, tool preview)
```

Single `<canvas>` element, one `requestAnimationFrame` render loop. All layers rendered in order per frame.

**Undo/redo:** State snapshot model. Each action saves a full copy of the annotations array. Undo restores the previous snapshot. Annotation data is small (coordinates, colors, text) — 100 undo steps of 20 annotations is negligible memory.

#### 4.4 Sidebar

- **Page name input** — optional label for the page/component being reviewed
- **General notes textarea** — page-level feedback not tied to a specific annotation (e.g., "this page isn't designed yet")
- **Annotation list** — numbered entries with comment preview, position info, click to highlight on canvas
- **Click annotation** in list → canvas pans and zooms to show it

#### 4.5 Git-Aware Context

On capture or image load, CodeEye auto-detects:
- **Project name** — from nearest `.git` remote URL (e.g., `standardpos`)
- **Branch** — current git branch (e.g., `dev`)
- **Working directory** — project root path
- **Suggested file** — auto-detect from active window (URL path → route file mapping for web apps, window title for editors). Falls back to manual input with autocomplete from project file tree.
- **Recent diff** (optional, toggleable) — last N lines of `git diff` (staged + unstaged), scoped to frontend files only, truncated to 200 lines max

Included in the exported prompt:
```
**Project:** standardpos (branch: dev)
**Likely file:** src/app/[locale]/(main)/returns/page.tsx
**Viewport:** 768 x 1024px
```

This lets the AI jump directly to the right file without asking.

#### 4.6 Export & Integrations

| Action | Description |
|--------|------------|
| Copy Prompt | Structured markdown to clipboard |
| Send to AI (MCP) | Direct delivery via MCP to connected AI tools |
| Copy Image | Annotated PNG to clipboard |
| Save Image | Download annotated PNG |
| Export JSON | Full session state (image + annotations + notes + git context) |
| Drag to AI Chat | Drag annotated image from export bar into any app; prompt auto-copies to clipboard |
| File GitHub Issue | Creates issue with annotated image + structured description |
| File Linear Issue | Creates issue with annotated image + structured description |

**Drag & drop outbound:**
- Pre-render `annotated.png` to disk on every annotation change (debounced)
- Drag from export bar initiates native file drag via Tauri's `startDragging` API
- Prompt text simultaneously copied to clipboard
- User drops image into browser/Slack/Discord, pastes text — works with any AI tool without MCP

**Prompt preview & edit:**
Before export, an optional preview step:
- Full rendered markdown shown in a panel
- All fields editable inline (file path, severity, comments)
- Free-text instruction line at the top for the AI (e.g., "don't change the header, just fix the table")
- Power users can toggle "auto-send without preview" in settings for fastest flow

**Prompt structure:**
```markdown
# UI Feedback Report

**Page/Component:** Return Items Page
**Project:** standardpos (branch: dev)
**Likely file:** src/app/[locale]/(main)/returns/page.tsx
**Viewport:** 768 x 1024px
**Screenshot:** return-items.png (1920 x 1080px)
**Issues Found:** 3

## General Feedback

The return items page is not properly designed. Layout feels cluttered and lacks visual hierarchy.

---

### Issue 1 [critical]: Search bar needs better styling
- **Where:** top area — approximately 35% from left, 18% from top (672px, 194px absolute)
- **Region size:** ~529 x 32px

### Issue 2 [minor]: Button should be full width
- **Where:** bottom-right area — approximately 87% from left, 82% from top (1670px, 886px absolute)
- **Region size:** ~159 x 37px

### Issue 3 [suggestion]: Add zebra striping to table rows
- **Where:** center area — approximately 50% from left, 50% from top (960px, 540px absolute)
- **Region size:** ~760 x 225px

---

*Please review and fix each issue above. The numbered annotations on the annotated screenshot correspond to these issues. Prioritize by severity: critical > minor > suggestion.*
```

**Coordinates:** Both percentage-based (for responsive context) and absolute pixel values (for precision) included per issue.

#### 4.7 Session History

- **Auto-save:** Every change auto-saves to `~/.codeeye/sessions/<id>/` (debounced 500ms)
- **Session contents:** original image, compressed image, annotated image, annotation state, notes, git context, tags, viewport, timestamp
- **Session list:** Collapsible panel — thumbnail, page name, date, annotation count, project badge
- **Project grouping:** Sessions organized by project (auto-detected from git remote)
- **Restore:** Click any session to reopen with all annotations and notes intact
- **Management:** Delete individual sessions, clear all

**Session tags & filtering:**
- Auto-tags: severity levels present (`has:critical`), status (`open`, `resolved`), project name
- User-defined custom tags (e.g., `homepage`, `checkout-flow`, `dark-mode`)
- Filter/search bar: combine tags, date range, annotation count
- Example: all unresolved critical issues on the checkout flow from this week

**Auto-cleaner:**
- Runs on app launch + daily interval while running
- Two retention rules — **whichever triggers first:**
  - Time-based: delete sessions older than X days (configurable, default 30)
  - Count-based: keep last N sessions (configurable, default 200)
- Cleans session images, prompt exports, and annotation data
- Logs deletions to `~/.codeeye/cleanup.log`

**Storage versioning:**
- `version` field in `index.json` and `meta.json`
- Migration runner on app launch: detects old versions, upgrades session format automatically
- Ensures forward compatibility when session structure changes between releases

#### 4.8 Batch Review Mode

Review an entire page in one session with multiple captures:
- Capture multiple regions in sequence without leaving capture mode
- Each capture becomes a separate annotated frame within one session
- Sidebar shows all frames as a scrollable list
- Single export generates one prompt covering all frames with cross-referenced issue numbers

```markdown
### Frame 1 — Header (1920x200px)
Issue #1 [critical]: Logo misaligned
Issue #2 [minor]: Nav spacing

### Frame 2 — Data Table (1920x600px)
Issue #3 [critical]: Columns overlap at 768px
```

One session, one prompt, full page review.

#### 4.9 Feedback Loop (Optional)

Re-capture and compare flow for verifying AI fixes. **Disabled by default**, user enables in settings.

When enabled:
- After sending feedback, session stays "open"
- User re-captures the same region after the AI's fix
- CodeEye shows before/after side-by-side
- User marks issues as resolved or adds follow-up annotations
- Updated feedback sent through MCP: "Issue #1 fixed, Issue #2 still broken, here's updated screenshot"

Turns CodeEye from a one-shot reporter into a feedback loop.

#### 4.10 MCP Integration Hub

**First launch — Welcome card:**
Auto-scans `$PATH` and known config locations for AI CLIs. Shows a non-blocking welcome card:

- Green `●` = detected and ready to connect
- Gray `○` = not found on system
- "Enable All Found" = one click, all tools configured
- "Skip" = close, configure later in Settings

**Supported tools (launch):**

| Tool | Config Location | Detection |
|------|----------------|-----------|
| Claude Code | `.mcp.json` or `~/.claude/settings.json` | `claude` in PATH |
| Codex (OpenAI) | Tool-specific config | `codex` in PATH |
| Gemini CLI | Tool-specific config | `gemini` in PATH |
| OpenCode | Tool-specific config | `opencode` in PATH |
| Aider | Tool-specific config | `aider` in PATH |
| Cline | VS Code extension path | Extension directory scan |
| Continue.dev | Tool-specific config | Extension directory scan |

**Settings — Integration Hub:**

Per-tool status and actions:

| Status | Badge | Actions |
|--------|-------|---------|
| Connected | `● Green` | Test Connection, Disconnect, Open Config |
| Available (detected, not configured) | `○ Yellow` | Connect (one-click) |
| Not Found | `— Gray` | Manual Setup Guide link |

Additional controls:
- **"+ Add Custom Tool"** — form for any MCP-compatible tool not auto-detected
- **"Copy MCP Config"** — raw JSON snippet for manual paste (universal fallback)
- **"Test Connection"** — sends MCP ping, verifies round-trip
- **MCP Server status** — running/stopped, restart button

**Config adapter architecture (Rust):**
```rust
trait McpConfigAdapter {
    fn detect(&self) -> ToolStatus;       // Installed? Configured?
    fn config_path(&self) -> PathBuf;     // Where to write
    fn connect(&self) -> Result<()>;      // Add CodeEye MCP entry
    fn disconnect(&self) -> Result<()>;   // Remove CodeEye MCP entry
    fn test(&self) -> Result<bool>;       // Verify connection works
}
```

Safety: Always backs up config files before modifying. Shows diff preview if file has existing MCP entries.

**MCP tools exposed to AI (read-only for v1):**

```rust
/// List sessions matching a filter
list_feedback_sessions(
    status: Option<String>,           // "open" | "resolved" | "all"
    project: Option<String>,          // filter by project name
    limit: Option<u32>,               // default 10
) -> Vec<SessionSummary>

/// Get full session detail
get_feedback_session(
    session_id: String,               // from list_feedback_sessions
) -> FeedbackSession                  // prompt + annotated image (base64) + metadata

/// Mark session as resolved
resolve_feedback_session(
    session_id: String,
    resolution_note: Option<String>,  // AI can describe what it fixed
) -> bool
```

**MCP server lifecycle:** Starts/stops with the Tauri app. Tray ensures the app is always running. If user quits CodeEye, AI tools get a broken pipe — standard MCP behavior.

**Registration:** When user clicks "Connect" in the Integration Hub, the Rust adapter writes CodeEye's MCP entry into that tool's config file. The entry points to the CodeEye binary as the stdio command.

#### 4.11 Image Compression Pipeline

Screenshots from high-DPI displays can be 5-10MB. Compression keeps exports fast and MCP payloads reasonable.

**Pipeline (Rust side):**
```
Raw capture (e.g., 3840x2160 PNG, 8MB)
  → Resize: cap longest edge at 1920px, preserve aspect ratio
    → Format: PNG for annotated export, WebP for MCP payload
      → Quality: configurable (default 85%)
        → Output: ~200-500KB
```

**When it runs:**
- On capture: store original at full resolution + generate compressed version
- On annotation change: re-render annotated PNG from compressed version (debounced)
- On export/MCP: use compressed version

**Both original and compressed images are kept.** Disk is cheap, sessions auto-clean, and the original is insurance for re-export at full quality.

**User controls in settings:**
- Max resolution (default 1920px)
- Format preference (PNG or WebP)
- Quality slider

#### 4.12 Viewport Metadata

When the capture source is a browser window:
- Auto-detect and include window dimensions in the prompt
- Included as: `**Viewport:** 768 x 1024px`
- Optional manual breakpoint label: `mobile`, `tablet`, `desktop`

Lets the AI check CSS at the specific breakpoint where the bug occurs.

---

### Tier 2 — Polish (post-launch, high value)

#### 4.13 Video/GIF Capture

For interaction bugs that static screenshots can't show (hover states, animations, transitions, loading spinners):

- **Record mode:** Toggle via toolbar button or `R` shortcut
- **Duration:** 3-10 second clips (configurable, default 5s)
- **Implementation:** Platform native APIs (macOS: ScreenCaptureKit, Windows: DXGI, Linux: PipeWire)
- **Output:** GIF or WebM (user preference)
- **Annotation on frames:** Pause playback, annotate specific frames, resume
- **Export:** Embedded in prompt as "See attached recording showing the hover state issue"
- **File size target:** <5MB per clip via quality/resolution controls

#### 4.14 Keyboard-First Workflow

Full annotation flow without the mouse:
- `Tab` / `Shift+Tab` cycle between annotations
- `Enter` to edit selected annotation's comment
- `1-9` jump to annotation by number
- `/` opens command palette (search tools, actions, sessions)

#### 4.15 Issue Tracker Integration

- **GitHub Issues:** OAuth flow or PAT, select repo, create issue with annotated image + structured markdown body
- **Linear:** API key, select team/project, create issue with labels
- **Templates:** Customizable issue templates per project

---

### Tier 3 — Future (community-driven)

- Browser extension for one-click page capture (Chrome, Firefox, Safari)
- URL-based capture via embedded webview
- Figma integration (import frames, compare implementation vs design)
- Custom prompt templates per AI provider
- Plugin system for annotation tools
- Team sharing via import/export JSON sessions
- Accessibility overlay (contrast ratios, missing alt text, focus order)
- Responsive testing (capture same page at multiple viewports)
- MCP write operations (AI triggers re-capture for verification)

---

## 5. Settings

### General
- **Global shortcut:** Configurable hotkey (default `Cmd+Shift+E` / `Ctrl+Shift+E`)
- **Launch at login:** Toggle auto-start
- **Theme:** Dark (default) / Light / System
- **Capture sound:** Toggle shutter sound on capture
- **Session limit:** Max sessions to keep (default 200)
- **Session retention:** Max age in days (default 30)
- **Feedback loop:** Enable/disable re-capture and compare (default: disabled)
- **Git diff in prompt:** Include recent diff in exports (default: enabled)
- **Prompt preview:** Show preview before export (default: enabled)
- **Image compression:** Max resolution, format, quality slider

### Quick Labels
- Manage built-in and custom annotation presets
- Set default severity per label

### Integrations
- MCP Integration Hub (Section 4.10)
- GitHub / Linear connections
- Custom tool configurations

### Danger Zone
- **Uninstall CodeEye** — clean removal of all traces:

Confirmation dialog with checkboxes:
```
☑ Disconnect all AI tools
  Remove MCP config from all connected tools

☑ Delete session history
  Remove ~/.codeeye/ (N sessions, X MB)

☑ Remove global shortcut
  Unregister Cmd+Shift+E

☐ Keep settings backup
  Save codeeye-backup.json before removing
  (lets you restore if you reinstall)

[Cancel]              [Uninstall and Quit]
```

Uninstall sequence:
1. Disconnect all AI tools (remove MCP entries via adapters)
2. Delete `~/.codeeye/` (sessions, settings, cache)
3. Unregister global shortcut
4. Remove system tray icon
5. Remove autostart entry
6. Optionally save backup JSON
7. Platform-specific app removal:
   - macOS: move `.app` to Trash
   - Windows: schedule deletion after quit
   - Linux: remove `.desktop` file and binary

---

## 6. Architecture

### 6.1 IPC Boundary

**Rust (Tauri backend) owns:**
- Screen capture + multi-monitor detection
- Global hotkey registration
- File I/O (session save/load, image read/write)
- Git context detection (project, branch, diff)
- MCP server + adapter management
- Image compression/resizing
- Auto-cleaner (scheduled + on-launch)
- System tray lifecycle
- Single instance enforcement

**Svelte (frontend) owns:**
- Canvas rendering + annotation engine
- All UI state (tools, selections, zoom/pan)
- Undo/redo stack (state snapshots)
- Prompt generation (assembling markdown from annotations + metadata)
- Drag & drop handling
- Keyboard shortcuts (annotation tools)
- Prompt preview & editing

**Prompt generation:** Frontend generates the prompt, requests git context and image data from Rust via a single IPC call. Frontend stitches it all together. Minimal IPC round-trips.

### 6.2 Annotation Data Model

```typescript
type Annotation = {
  id: string
  type: 'circle' | 'rectangle' | 'arrow' | 'freehand' | 'text'
  number: number                    // #1, #2, etc.
  frame: number                     // which frame in batch mode (0 for single)
  bounds: { x: number, y: number, w: number, h: number }
  points?: { x: number, y: number }[]  // freehand path, arrow start/end
  label: string                     // quick label or custom text
  comment: string                   // detailed comment from popup
  severity: 'critical' | 'minor' | 'suggestion'
  color: string                     // hex, default #f97316
  created_at: number                // timestamp
}

type Session = {
  id: string
  version: number                   // storage format version for migrations
  status: 'open' | 'resolved'
  page_name: string
  general_notes: string
  tags: string[]
  frames: Frame[]
  annotations: Annotation[]
  git: GitContext
  viewport: { width: number, height: number } | null
  settings: { show_preview: boolean, include_diff: boolean }
  created_at: number
  updated_at: number
}

type Frame = {
  index: number
  original_path: string
  compressed_path: string
  annotated_path: string
  dimensions: { width: number, height: number }
}

type GitContext = {
  project: string
  branch: string
  working_directory: string
  suggested_file: string | null
  recent_diff: string | null        // truncated, frontend files only
}
```

### 6.3 Storage Structure

```
~/.codeeye/
├── config.json                    # App settings, preferences
├── sessions/
│   ├── index.json                 # Lightweight manifest (id, project, tags, date, status)
│   └── <session-id>/
│       ├── meta.json              # Git context, viewport, annotations, notes, tags
│       ├── original.png           # Raw screenshot (unmodified, full resolution)
│       ├── compressed.png         # Resized for export
│       ├── annotated.png          # Rendered with annotations (for drag & drop)
│       └── frames/                # Batch mode: multiple captures
│           ├── 01-original.png
│           ├── 01-compressed.png
│           ├── 01-annotated.png
│           └── ...
├── logs/                          # Application logs
│   ├── codeeye.log               # Current log file
│   └── codeeye.log.1             # Rotated (max 5 files, 5MB each)
└── cleanup.log                    # Auto-cleaner run history
```

### 6.4 Error Handling

**Recoverable (user sees a toast, app continues):**
- Git context detection fails → proceed without git context
- Session save fails → retry once, then toast
- MCP adapter can't find tool config → mark as "Not Found"
- Image compression fails → fall back to original uncompressed
- Auto-cleaner fails → log, skip this run, try next launch
- Clipboard read/write fails → toast "Clipboard unavailable"
- Canvas rendering error → skip frame, log to console

**Fatal (error screen with guidance):**
- Screen capture API unavailable → "Check permissions" guidance
- Storage directory not writable → "Can't write to storage directory"

**Core principle:** Never crash the app. Degrade gracefully. Missing git context or failed compression should never block annotating and exporting.

**Invalid annotation state:** Restore to last known good snapshot from undo stack.

### 6.5 Capture Flow

```
User presses Ctrl+Shift+E
  → Rust: detect which monitor has cursor
    → Rust: spawn transparent fullscreen window on that display (dim overlay)
      → Svelte: RegionOverlay renders crosshair + drag selection
        → Mouse release: keyboard resize phase (arrows, live dimensions)
          → Enter: Rust captures region at native resolution
            → Rust: store original, generate compressed version
              → Rust: detect viewport size if browser window
                → Rust: gather git context (project, branch, diff)
                  → Svelte: editor opens with capture loaded
```

### 6.6 Drag & Drop

**Inbound:**
- File drop → Tauri `drag-drop` event → read image → compression pipeline → canvas
- Clipboard paste → Rust reads clipboard image → compression pipeline → canvas
- File browser → Tauri `dialog` API → read file → compression pipeline → canvas

**Outbound:**
- On annotation change (debounced): pre-render `annotated.png` to disk
- User drags from export bar → Rust initiates native file drag via `startDragging`
- Prompt text simultaneously copied to clipboard
- Works with any app: browser tabs, Slack, Discord, etc.

---

## 7. Project Structure

```
codeeye/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # App entry, tray, global shortcuts, single instance
│   │   ├── capture.rs            # Screen capture + multi-monitor + region select
│   │   ├── compression.rs        # Image resize + format conversion pipeline
│   │   ├── mcp/
│   │   │   ├── mod.rs            # MCP stdio server (embedded)
│   │   │   └── adapters/         # Per-tool config adapters
│   │   │       ├── claude.rs
│   │   │       ├── codex.rs
│   │   │       ├── gemini.rs
│   │   │       └── ...
│   │   ├── git.rs                # Git context detection + diff
│   │   ├── storage.rs            # Session persistence + migrations
│   │   ├── cleaner.rs            # Auto-cleanup (time + count based)
│   │   ├── permissions.rs        # OS permission checks (screen capture)
│   │   └── uninstall.rs          # Clean removal logic
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # Svelte 5 frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Canvas.svelte            # Annotation canvas (Canvas 2D) + zoom/pan
│   │   │   ├── Toolbar.svelte           # Tool selection bar + quick labels
│   │   │   ├── Sidebar.svelte           # Annotations list + notes + frame list
│   │   │   ├── CommentPopup.svelte      # Inline comment editor
│   │   │   ├── PromptPreview.svelte     # Editable prompt preview before export
│   │   │   ├── SessionList.svelte       # History panel with project groups + tags + search
│   │   │   ├── RegionOverlay.svelte     # Screen capture region selector + keyboard resize
│   │   │   ├── FeedbackLoop.svelte      # Before/after compare (optional feature)
│   │   │   ├── WelcomeCard.svelte       # First-launch integration setup
│   │   │   ├── Settings.svelte          # Full settings page
│   │   │   └── IntegrationHub.svelte    # MCP tool management
│   │   ├── state/                       # Svelte 5 runes (.svelte.ts files)
│   │   │   ├── annotations.svelte.ts    # Annotation state + undo/redo snapshots
│   │   │   ├── canvas.svelte.ts         # Zoom, pan, viewport transforms
│   │   │   ├── sessions.svelte.ts       # Session history + project index
│   │   │   ├── tools.svelte.ts          # Active tool, shape, color, quick labels
│   │   │   └── git.svelte.ts            # Git context (project, branch, diff)
│   │   ├── i18n/                        # Internationalization
│   │   │   ├── index.ts                 # i18n setup + locale loader
│   │   │   └── locales/
│   │   │       └── en.json              # English (default, only language at launch)
│   │   └── utils/
│   │       ├── geometry.ts       # Hit-testing, transforms, bounding boxes (pure TS)
│   │       ├── export.ts         # Prompt generation, PNG/JSON export
│   │       └── shortcuts.ts      # Keyboard shortcut registry
│   ├── App.svelte
│   └── main.ts
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── README.md
└── LICENSE                       # MIT
```

---

## 8. Build & Release

**Local (pre-commit):**
```
ultracite (lint + format Svelte/TS)
cargo fmt --check + cargo clippy
cargo test
vitest run
```

**GitHub Actions (tag push only):**
```
Tag v1.0.0
  → Build all targets in parallel (Tauri Action)
    ├── macOS (Intel + ARM) → .dmg
    ├── Windows → .msi + .exe
    └── Linux → .deb, .rpm, .AppImage
  → Upload artifacts to GitHub Release
  → Generate release notes from commits
```

**Code signing:**
- macOS: Apple Developer cert + notarization (required)
- Windows: code signing cert (optional for v1, SmartScreen warning without)
- Linux: not needed

**Auto-update:** Tauri updater plugin checks GitHub Releases. Non-blocking "Update available" in tray. One-click update.

---

## 9. Success Metrics

| Metric | Target |
|--------|--------|
| Hotkey to sent-feedback | < 30 seconds |
| App cold start | < 1 second |
| Binary size | < 10MB |
| Idle RAM | < 20MB |
| MCP setup time | < 10 seconds (one-click) |
| Session restore | < 500ms |
| Image compression | < 500KB per export |

---

## 10. Non-Goals (v1)

- Not a design system or component library
- Not a multi-user real-time collaboration tool
- Not a browser extension (Tier 3)
- Not a full project management tool
- No server-side storage — everything stays local
- No account or sign-up required
- No project-level config files (`.codeeye.json`) — may revisit post-launch
- No component detection / source map integration
- No browser console/network capture
- MCP is read-only — no AI-triggered captures
- No multi-window — single window, session switching via session list
- No telemetry, no analytics, no tracking

---

## 11. Resolved Questions

| # | Question | Decision | Rationale |
|---|----------|----------|-----------|
| 1 | MCP server: sidecar or embedded? | Embedded | Single process, simpler. Tray ensures always running. |
| 2 | GitHub/Linear OAuth flow? | Deferred to Tier 2 | Not in v1 scope. |
| 3 | Video/GIF capture approach? | Platform native APIs, Tier 2 | Deferred from v1 to reduce scope. |
| 4 | Project-level config (`.codeeye.json`)? | No for v1 | Keep it simple. Revisit post-launch. |
| 5 | Coordinate system? | Both percentage + absolute pixels | Percentage for responsive context, absolute for precision. |
| 6 | MCP API naming? | `list/get/resolve_feedback_sessions` | Clear noun, no abbreviations. |
| 7 | Auto-cleaner trigger? | On launch + daily | Whichever retention rule hits first (time or count). |
| 8 | Original image retention? | Keep both original + compressed | Disk is cheap, auto-cleaner handles limits. |
| 9 | MCP concurrency with UI? | Serve last-saved state | No locking — stale data is acceptable for simplicity. |
| 10 | Sensitive data in screenshots? | User's responsibility | Document in docs, no in-app detection. |
| 11 | Multilanguage? | i18n-ready, English only at launch | Wire i18n plumbing from day one. Adding languages = adding JSON files. |
| 12 | Landing page? | No for v1 | GitHub repo + README only. |
| 13 | License? | MIT | Maximum adoption, companies can use freely. |
| 14 | Multi-window? | Single window | One session at a time, switch via session list. Simpler. |
| 15 | Telemetry? | None | No analytics, no tracking. Explicit in README. |

---

## 12. Brand Identity

### Icon Concept

Eye shape formed by `< >` angle brackets — developer-oriented, communicates "code" + "eye" visually. Works at all sizes from 16x16 tray icon to 1024x1024 source.

**Required sizes:**
- 16x16, 32x32 (system tray / tray @2x)
- 128x128 (app list)
- 512x512 (installer)
- 1024x1024 (source asset)

### Color Palette

| Role | Color | Usage |
|---|---|---|
| Primary | `#f97316` (orange) | Icon, accents, active states, default annotation color |
| Background Dark | `#0a0a0a` | App background (dark theme) |
| Background Light | `#fafafa` | App background (light theme) |
| Surface Dark | `#171717` | Cards, panels (dark theme) |
| Surface Light | `#f5f5f5` | Cards, panels (light theme) |
| Text Primary | `#fafafa` / `#0a0a0a` | Dark / light theme |
| Text Muted | `#737373` | Secondary text, hints |
| Border | `#262626` / `#e5e5e5` | Dark / light theme |
| Critical | `#ef4444` (red) | Critical severity annotations |
| Minor | `#eab308` (yellow) | Minor severity annotations |
| Suggestion | `#3b82f6` (blue) | Suggestion severity annotations |

### Typography

System font stack — no custom fonts. Fast load, native feel per platform.

### Assets Needed

- App icon (all sizes above)
- Installer graphics: macOS `.dmg` background, Windows installer banner
- Welcome card graphic (first-launch screen)
- GitHub repo social preview image (1280x640)

---

## 13. Internationalization (i18n)

**Strategy:** i18n-ready from day one, ship English only at launch.

- All user-facing strings extracted to locale files from the start
- Locale files stored as JSON (e.g., `src/lib/i18n/en.json`)
- Adding a new language = adding one JSON file + registering it
- No RTL support needed for v1 (revisit when Arabic/Hebrew requested)
- MCP tool descriptions and prompt templates remain English (developer-facing, AI-consumed)
- Settings include language picker (shows only English for v1, ready for expansion)

---

## 14. Logging & Diagnostics

**Log location:** `~/.codeeye/logs/`

- Structured log output (timestamp, level, module, message)
- Log levels: `error`, `warn`, `info`, `debug` (configurable in settings, default `info`)
- Rotation: max 5 files, 5MB each. Oldest deleted when limit reached.
- Logs cover: capture events, MCP requests/responses, session save/load, adapter operations, error details

**Export for bug reports:**
- Settings → "Export Diagnostic Bundle" button
- Generates a `.zip` with: last 3 log files, `config.json` (sensitive paths redacted), app version, OS version, display info
- User attaches this to GitHub Issues when reporting bugs

**What is NOT logged:**
- Screenshot content or image data
- Annotation text or comments
- Git diffs or file paths (only project name + branch)
- No logs are ever sent anywhere — local only

---

## 15. Global Hotkey Conflict Handling

- On startup, attempt to register the configured hotkey (default `Cmd+Shift+E` / `Ctrl+Shift+E`)
- If registration fails (another app owns the shortcut):
  1. Toast: "Hotkey Ctrl+Shift+E is in use by another app"
  2. CodeEye still launches and works — just no global hotkey
  3. User can capture via tray menu → "New Capture" or opening the app directly
  4. Toast includes "Change shortcut" link → opens Settings → shortcut config
- Settings shows a "Test" button that verifies the shortcut is registered and working
- Common conflict-free alternatives suggested: `Ctrl+Shift+F12`, `Ctrl+Alt+E`

---

## 16. First-Run Experience

Full first launch flow (runs once, never again):

```
App launches for the first time
  │
  ▼
Step 1: Permission check (macOS only)
  "CodeEye needs screen recording permission"
  → Guide user to System Settings → Privacy → Screen Recording
  → Wait for permission granted, then continue
  │
  ▼
Step 2: Quick intro overlay (3 panels, skippable)
  Panel 1: "Press Ctrl+Shift+E anywhere to capture"
  Panel 2: "Draw on the screenshot to highlight issues"
  Panel 3: "Send to your AI assistant with one click"
  → "Get Started" / "Skip"
  │
  ▼
Step 3: MCP Integration Welcome Card (Section 4.10)
  Auto-scan for AI tools
  → "Enable All Found" / "Skip"
  │
  ▼
Step 4: Ready
  Empty state with clear CTA: "Press Ctrl+Shift+E to capture your first screenshot"
```

- First-run state tracked in `config.json` (`"first_run_complete": true`)
- Each step can be revisited from Settings (re-run tour, re-scan integrations)

---

## 17. Privacy & Telemetry

**Explicit policy (stated in README and Settings → About):**

- CodeEye collects **zero** telemetry, analytics, or usage data
- No network calls except:
  - Auto-update check (GitHub Releases API, can be disabled)
  - MCP communication (local stdio, no network)
  - GitHub/Linear API (Tier 2, user-initiated only)
- All data stays in `~/.codeeye/` on the user's machine
- Screenshots may contain sensitive information — this is the user's responsibility
- No account, no sign-up, no cloud, no server

---

## 18. README Structure

GitHub repo README covers:

1. **Hero** — logo, one-liner tagline, screenshot/GIF of the capture→annotate→send flow
2. **Features** — bullet list with icons, brief descriptions
3. **Quick Start** — install (brew/apt/winget/download), first capture in 3 steps
4. **MCP Setup** — one-click via app, or manual config snippet for each supported tool
5. **Keyboard Shortcuts** — full table of annotation + capture shortcuts
6. **How It Works** — brief architecture overview (Tauri + Svelte + Rust)
7. **Building from Source** — prerequisites, clone, build, run
8. **Contributing** — code of conduct, PR process, development setup
9. **Privacy** — "Zero telemetry" statement, link to full policy
10. **License** — MIT
