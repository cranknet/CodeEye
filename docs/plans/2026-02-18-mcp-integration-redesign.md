# MCP Integration Redesign

**Date:** 2026-02-18
**Status:** Design — awaiting implementation plan
**Scope:** Fix the broken MCP pipeline, add payload settings, redesign Integration Hub

---

## Problem Statement

The MCP integration is structurally complete but functionally broken. AI tools can connect and see the three MCP tools (`list_ui_feedback`, `get_ui_feedback`, `resolve_ui_feedback`), but calling them returns empty results because:

1. No session is ever saved to disk during the capture/annotate/export flow
2. The "Send to AI" button only copies text to clipboard — identical to "Copy Prompt"
3. The generated prompt omits git context
4. The MCP server returns images as text blobs instead of proper MCP image content
5. Users have no control over what data the MCP server exposes

Additionally, the Integration Hub is a minimal list UI that needs a full redesign with per-tool configuration, backup management, and uninstall options.

---

## Issues Identified

| #  | Issue | Severity | Category |
|----|-------|----------|----------|
| 1  | Session never created during capture flow | Critical | Flow broken |
| 2  | Screenshot image never saved to disk | Critical | Flow broken |
| 3  | Rendered prompt not persisted (`prompt.md`) | Critical | Flow broken |
| 4  | MCP `get_ui_feedback` doesn't return prompt | Critical | Flow broken |
| 5  | `generatePrompt()` caller doesn't pass gitContext | High | Data gap |
| 6  | "Send to AI" button label is misleading | High | UX |
| 7  | "Send to AI" and "Copy Prompt" are identical actions | Medium | UX |
| 8  | `AppConfig` has no MCP payload fields | Medium | Feature gap |
| 9  | Prompt template is hardcoded in `export.ts` | Medium | Feature gap |
| 10 | No settings UI for MCP payload configuration | Medium | Feature gap |
| 11 | Missing `export_diagnostic_bundle` Tauri command (runtime crash) | Medium | Bug |
| 12 | ExportBar has no session/save awareness | Medium | Refactor |
| 13 | Session restore doesn't load the screenshot image | Medium | Flow broken |
| 14 | MCP image should use `type: "image"` content blocks | Medium | MCP compliance |
| 15 | No `annotated.png` rendering (annotations burned onto screenshot) | Medium | Feature gap |
| 16 | MCP server has no stderr logging | Low | Observability |
| 17 | Compression ignores quality param / WebP not supported | Low | Config ignored |
| 18 | No git context attached to sessions at creation | Medium | Data gap |
| 19 | Settings "Integrations" section is a dead-end placeholder | Medium | Feature gap |
| 20 | `ToolContent` struct doesn't support image type | Medium | MCP compliance |
| 21 | Integration Hub needs redesign with detail views | Medium | UX |

---

## Design

### 1. Session Persistence Pipeline

The core fix — making sessions save to disk so MCP can serve them.

**Current flow:**
```
Capture -> in-memory image -> annotate -> "Send to AI" -> clipboard copy -> gone
```

**New flow:**
```
Capture -> create session on disk -> annotate (auto-save) -> "Submit Feedback" -> save image + prompt + meta -> MCP serves it
```

#### Step-by-step behavior

**On capture** (`startCapture` in `App.svelte`):
- After getting the base64 image from `capture_screen`, call `sessionState.createSession(pageName, project)` to get a session ID
- Call `save_session_capture(session_id, base64)` to write `original.png` and `compressed.png` to disk
- Call `get_git_context` with the current working directory, merge into session meta
- Every capture is now persisted immediately, not just on export

**During annotation** (existing `scheduleSave` debounce):
- Wire the debounced save to fire whenever `annotations`, `pageName`, or `generalNotes` change via `$effect`
- This keeps `meta.json` up to date as the user works

**On "Submit Feedback"** (primary button):
- Render prompt markdown using the template engine
- Save `prompt.md` to the session directory
- Render annotated image via `canvas.toDataURL()` and save as `annotated.png`
- Update session meta with `submitted_at` timestamp
- Copy prompt to clipboard as a convenience
- Show toast: "Feedback submitted — available via MCP"

**Session restore** (`handleSessionRestore`):
- Add a new Tauri command `load_session_image(session_id: String) -> Result<String, String>` that returns base64 of the best available image (compressed > annotated > original)
- Call it during restore to populate the canvas

#### Files changed

| File | Change |
|------|--------|
| `src/App.svelte` | Wire session creation into `startCapture`, add git context fetch, add submit handler, pass new props to ExportBar |
| `src/lib/state/sessions.svelte.ts` | Add `$effect` to trigger `scheduleSave` on state changes |
| `src-tauri/src/lib.rs` | Add `load_session_image` command |
| `src-tauri/src/storage.rs` | Add `load_session_image` function |

---

### 2. MCP Server Improvements

#### A. Return prompt markdown in `get_ui_feedback`

Read `prompt.md` from the session directory and include it in the response:

```json
{
  "session_id": "abc-123",
  "meta": { ... },
  "prompt": "# UI Feedback Report\n...",
  "image_base64": "iVBOR..."
}
```

The prompt field is the primary content — a ready-to-act description the AI tool can use immediately.

#### B. Proper MCP image content blocks

Instead of embedding base64 inside a JSON text blob, return a separate content item with `type: "image"`:

```json
{
  "content": [
    { "type": "text", "text": "{ session_id, meta, prompt }" },
    { "type": "image", "data": "iVBOR...", "mimeType": "image/png" }
  ]
}
```

Extend `ToolContent` in `protocol.rs` to support both variants:

```rust
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ToolContent {
    Text {
        #[serde(rename = "type")]
        content_type: String,  // "text"
        text: String,
    },
    Image {
        #[serde(rename = "type")]
        content_type: String,  // "image"
        data: String,          // base64
        #[serde(rename = "mimeType")]
        mime_type: String,     // "image/png"
    },
}
```

#### C. Stderr logging

Add `eprintln!` calls in `run_server()` for:
- Server startup: `"codeeye-mcp started, reading from stdin"`
- Each request: `"<- {method} (id: {id})"`
- Errors: `"ERROR: {description}"`

Stderr doesn't interfere with the stdio JSON-RPC protocol on stdout.

#### D. Respect payload config

When `call_tool` runs, load `config.json` and check:
- `mcp_include_prompt` — include/exclude the `prompt` field
- `mcp_include_image` — include/exclude the image content block
- `mcp_include_metadata` — include/exclude the `meta` field
- `mcp_image_max_resolution` — re-compress image to this size if needed (may differ from stored compressed.png)

#### Files changed

| File | Change |
|------|--------|
| `src-tauri/src/mcp/tools.rs` | Read `prompt.md`, respect config toggles, return image as separate content block |
| `src-tauri/src/mcp/protocol.rs` | Change `ToolContent` to enum supporting text and image |
| `src-tauri/src/mcp/mod.rs` | Add stderr logging |

---

### 3. Config & Settings

#### A. New fields in `AppConfig`

```rust
// Add to AppConfig struct
pub mcp_include_prompt: bool,        // default: true
pub mcp_include_image: bool,         // default: true
pub mcp_include_metadata: bool,      // default: false
pub mcp_image_max_resolution: u32,   // default: 1280
pub mcp_prompt_template: String,     // default: DEFAULT_PROMPT_TEMPLATE
```

Flat fields in `AppConfig` — matches the existing pattern, avoids migration complexity.

MCP image resolution is intentionally separate from capture compression resolution. Store locally at 1920px, send to AI at 1280px.

Default for `mcp_prompt_template`:

```markdown
# UI Feedback Report

**Page:** {{page_name}}
**Project:** {{project}}
**Branch:** {{branch}}
**File:** {{suggested_file}}
**Viewport:** {{viewport}}

## Notes

{{notes}}

## Issues

{{annotations}}

## Recent Changes

{{recent_diff}}
```

#### B. Prompt template engine

Replace the hardcoded `generatePrompt()` in `export.ts` with a template renderer.

Available variables:

| Variable | Source |
|----------|--------|
| `{{page_name}}` | Sidebar input |
| `{{project}}` | Git context |
| `{{branch}}` | Git context |
| `{{suggested_file}}` | Git context |
| `{{notes}}` | General notes textarea |
| `{{annotations}}` | Auto-formatted annotation list (uses existing `formatAnnotation()` logic) |
| `{{recent_diff}}` | Git diff (if enabled in settings) |
| `{{viewport}}` | Image dimensions as `WxH` |

Implementation: simple `String.replace()` pass for each variable. No loops, no conditionals — not a full Handlebars parser. The `{{annotations}}` variable is special: it expands into the formatted annotation list using existing logic. Empty variables are removed (the section heading and the variable line both disappear).

The engine lives in a new function:

```typescript
// export.ts
export function renderTemplate(template: string, input: PromptInput): string
```

`generatePrompt()` becomes a wrapper that calls `renderTemplate` with the default template for backward compatibility.

#### C. Fix WebP/quality gap

In `compression.rs`, branch on format:
- `"png"` — current lossless behavior (quality parameter ignored, as expected)
- `"webp"` — use `ImageFormat::WebP` with the quality parameter

This makes the existing Settings UI quality slider actually work when WebP is selected.

#### D. Settings UI — new "MCP Output" section

Add a fifth section to the Settings nav:

```
◎ General
⊞ Labels
⊕ Integrations
⊛ MCP Output      <- new
⚙ Advanced
```

Contents:
- **Three toggles:** Include prompt / Include image / Include structured metadata
- **MCP image resolution dropdown:** 800px, 1280px (default), 1920px
- **Prompt template editor:** `<textarea>` pre-filled with the current template
- **"Reset to Default" button** next to the template editor
- **Variable reference:** small collapsible list showing available `{{variables}}`

#### Files changed

| File | Change |
|------|--------|
| `src-tauri/src/storage.rs` | Add MCP fields to `AppConfig` with defaults |
| `src/lib/utils/export.ts` | Add `renderTemplate()`, refactor `generatePrompt()` to use it |
| `src/lib/components/Settings.svelte` | Add "MCP Output" section with toggles, resolution picker, template editor |
| `src-tauri/src/compression.rs` | Support WebP format with quality parameter |

---

### 4. UI/UX Fixes

#### A. Redesign ExportBar

Current: four confusing buttons (Preview, JSON, Copy Prompt, Send to AI).

New: three clear buttons.

```
[ Preview ]                              [ Copy Prompt ]  [ Submit Feedback ]
                                                          ↑ primary orange CTA
```

- **Preview** — opens PromptPreview modal (unchanged)
- **Copy Prompt** — copies markdown to clipboard, shows "Copied!" feedback
- **Submit Feedback** — the primary button that triggers the full save pipeline

Drop the JSON button (unused, low value).

#### B. ExportBar new props

```typescript
interface Props {
  promptMarkdown: string;
  onshowpreview: () => void;
  onsubmit: () => void;       // new — triggers save pipeline
  submitting: boolean;         // new — disables button during save
}
```

The save logic lives in `App.svelte` (access to all stores). ExportBar fires the callback.

Button states:
- Default: "Submit Feedback"
- During save: "Submitting..." (disabled)
- After success: "Submitted!" (briefly, then resets)

#### C. PromptPreview button rename

"Copy & Send" -> "Copy". Honest about what it does. The submit action belongs in ExportBar.

#### D. Fix `export_diagnostic_bundle`

Add a Tauri command that:
1. Creates a temp directory
2. Copies `~/.codeeye/config.json` and `~/.codeeye/logs/*` into it
3. Zips them
4. Opens a save dialog for the user to choose where to save

No screenshots included — privacy safe.

#### E. Annotated PNG rendering

Render on the frontend (not Rust) — the Canvas already draws annotations.

On submit:
1. Call a method on `canvasState` that renders the current view to a data URL via `canvas.toDataURL("image/png")`
2. Send the base64 to Rust via a new command `save_annotated_image(session_id, base64)`
3. Rust writes it as `annotated.png` in the session directory

This avoids duplicating rendering logic in Rust and leverages existing Canvas code.

#### Files changed

| File | Change |
|------|--------|
| `src/lib/components/ExportBar.svelte` | Redesign to three buttons, add `onsubmit`/`submitting` props |
| `src/lib/components/PromptPreview.svelte` | Rename button to "Copy" |
| `src/lib/state/canvas.svelte.ts` | Add `exportToDataUrl()` method |
| `src-tauri/src/lib.rs` | Add `export_diagnostic_bundle`, `save_annotated_image` commands |
| `src/App.svelte` | Implement `handleSubmitFeedback()` orchestrating the full save pipeline |

---

### 5. Integration Hub Redesign

Replace the flat list modal with a two-panel management interface.

#### A. Layout

```
┌──────────────────────────────────────────────────────────┐
│  Integrations                                 Rescan  ✕  │
├───────────────┬──────────────────────────────────────────┤
│               │                                          │
│ Claude Code   │  Claude Code                             │
│ ● Connected   │  ─────────────────────────────           │
│               │  Status: Connected                       │
│ Gemini CLI    │  Config: ~/.claude.json                  │
│ ○ Installed   │  Binary: /home/.../codeeye               │
│               │                                          │
│ Codex         │  ┌─ MCP Entry ────────────────────┐     │
│ ○ Installed   │  │ "codeeye": {                    │     │
│               │  │   "type": "stdio",              │     │
│ OpenCode      │  │   "command": "/path/codeeye",   │     │
│ ◌ Not found   │  │   "args": ["--mcp"]             │     │
│               │  │ }                               │     │
│ Cursor        │  └────────────────────────────────┘     │
│ ◌ Not found   │                                          │
│               │  [ Verify ]  [ Disconnect ]               │
│               │                                          │
│               │  ┌─ Backup ──────────────────────┐      │
│               │  │ Backed up: ~/.claude.json.bak  │      │
│               │  │ [ Restore Backup ]             │      │
│               │  └────────────────────────────────┘     │
│               │                                          │
│               │  ┌─ Danger ──────────────────────┐      │
│               │  │ [ Uninstall MCP ]              │      │
│               │  │ Removes entry + deletes backup  │      │
│               │  └────────────────────────────────┘     │
├───────────────┴──────────────────────────────────────────┤
│  MCP stdio protocol · 5 tools scanned                     │
└──────────────────────────────────────────────────────────┘
```

#### B. Left panel — tool list

Each tool card shows:
- Tool name
- Status dot: green (connected), yellow (installed, not connected), gray (not found)
- Clicking a tool selects it and shows the detail view on the right
- First tool auto-selected on open

#### C. Right panel — detail view (context-dependent)

**Not installed:**
- Message: "{tool} not detected on this system"
- Installation hint (e.g., "Install Claude Code from claude.ai/code")

**Installed, not connected:**
- "Connect" button (primary orange)
- Config path preview: "Will write to `~/.claude.json`"
- MCP entry preview: read-only code block showing what will be injected

**Connected:**
- Status info: config path, binary path
- **MCP Entry** — read-only code block showing the injected JSON/TOML entry
- **Verify** button — runs adapter `verify()`, shows result inline. For tools without CLI verification (Gemini, Cursor), shows guidance text
- **Disconnect** button — removes the codeeye entry, preserves everything else
- **Backup section** — visible if `.bak` file exists. Shows backup path and a "Restore Backup" button
- **Uninstall MCP** — red danger section. Disconnect + delete backup. Confirmation dialog required: "This will remove CodeEye from {tool} and delete the config backup. Continue?"

#### D. New backend support

New trait methods on `McpConfigAdapter`:

```rust
fn backup_path(&self) -> Option<PathBuf>;
fn restore_backup(&self) -> Result<(), String>;
fn uninstall(&self) -> Result<(), String>;    // disconnect + remove .bak
fn read_entry(&self) -> Result<String, String>; // formatted MCP entry string
```

New Tauri commands:

```rust
restore_config_backup(tool_name: String) -> Result<(), String>
uninstall_mcp(tool_name: String) -> Result<(), String>
check_backup_exists(tool_name: String) -> Result<Option<String>, String>
read_mcp_entry(tool_name: String) -> Result<String, String>
```

#### E. WelcomeCard unchanged

The first-run wizard keeps its three-step flow (Welcome -> How it works -> Connect tools). Quick connect buttons only — detailed management belongs in the Integration Hub.

#### Files changed

| File | Change |
|------|--------|
| `src/lib/components/IntegrationHub.svelte` | Full rewrite — two-panel layout with detail views |
| `src-tauri/src/mcp/adapters/mod.rs` | Add `backup_path`, `restore_backup`, `uninstall`, `read_entry` to trait with default impls |
| `src-tauri/src/mcp/adapters/claude.rs` | Implement new trait methods |
| `src-tauri/src/mcp/adapters/gemini.rs` | Implement new trait methods |
| `src-tauri/src/mcp/adapters/codex.rs` | Implement new trait methods |
| `src-tauri/src/mcp/adapters/opencode.rs` | Implement new trait methods |
| `src-tauri/src/mcp/adapters/cursor.rs` | Implement new trait methods |
| `src-tauri/src/lib.rs` | Add `restore_config_backup`, `uninstall_mcp`, `check_backup_exists`, `read_mcp_entry` commands |

---

## Storage Layout (after changes)

```
~/.codeeye/
├── config.json                    # now includes mcp_* fields
├── sessions/
│   ├── index.json
│   └── <session-id>/
│       ├── meta.json              # annotations, git context, timestamps
│       ├── original.png           # full resolution capture
│       ├── compressed.png         # resized for storage
│       ├── annotated.png          # screenshot with burned-in annotations (new)
│       └── prompt.md              # rendered prompt markdown (new)
└── logs/
```

---

## Implementation Order

Recommended sequence (dependencies flow downward):

1. **Session persistence pipeline** (Section 1) — unblocks everything else
2. **Config fields + template engine** (Section 3A, 3B) — needed by MCP and UI
3. **MCP server improvements** (Section 2) — depends on persisted sessions + config
4. **UI/UX fixes** (Section 4) — ExportBar redesign, submit flow
5. **Settings UI** (Section 3D) — MCP Output section
6. **Integration Hub redesign** (Section 5) — independent, can parallel with 3-4
7. **Polish** (Section 3C WebP fix, Section 4D diagnostics, Section 2C logging)

---

## Out of Scope (v2)

- Push notifications via MCP `notifications/resources/updated`
- Multiple named prompt templates (Bug Report, Design Review, Accessibility Audit)
- Per-tool MCP payload overrides
- Loop/conditional syntax in prompt templates
- MCP write tools (AI triggering captures)
