# CodeEye

**The fastest way to report UI bugs to AI.**

CodeEye is a Tauri v2 desktop app that lets developers capture screenshots, annotate UI issues, and send structured feedback to AI coding assistants — all without leaving your workflow.

---

## Features

- **One-key capture** — press `Ctrl+Shift+E` (or `Cmd+Shift+E` on macOS) from anywhere to capture your screen
- **Annotation tools** — circle, rectangle, arrow, freehand, and text overlays with severity levels (critical / minor / suggestion)
- **Structured prompts** — auto-generated markdown with percentage + pixel coordinates per annotation, git context, and viewport size
- **MCP integration** — embedded stdio MCP server so AI tools (Claude Code, Codex, Gemini CLI) can read your feedback directly
- **Session history** — all captures saved to `~/.codeeye/` with tags, notes, and project grouping
- **Zero telemetry** — no network calls, no analytics, fully local

---

## Quick Start

### Download

Download the latest release for your platform from [GitHub Releases](https://github.com/cranknet/CodeEye/releases).

| Platform | Installer |
|----------|-----------|
| Linux    | `.AppImage` or `.deb` |
| macOS    | `.dmg` |
| Windows  | `.msi` or `.exe` |

### First Run

1. Launch CodeEye — the app appears in your system tray
2. Grant screen recording permission if prompted (macOS only)
3. Press `Ctrl+Shift+E` to capture your first screenshot
4. Annotate issues, add notes, and copy the prompt to your AI tool

---

## MCP Setup

CodeEye runs an embedded MCP server that AI tools can query for UI feedback.

### Claude Code

Add to your `~/.claude/mcp.json` (or use the Integration Hub in Settings):

```json
{
  "mcpServers": {
    "codeeye": {
      "command": "/path/to/codeeye",
      "args": ["--mcp"]
    }
  }
}
```

### Available MCP Tools

| Tool | Description |
|------|-------------|
| `list_feedback_sessions` | Query sessions by status, project, or limit |
| `get_feedback_session` | Get full session with base64 screenshot and annotations |
| `resolve_feedback_session` | Mark a session as resolved |

Use the **Integration Hub** (Settings → Integrations) for one-click setup with Claude Code, Codex, and Gemini CLI.

---

## Keyboard Shortcuts

### Global (system-wide)

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+E` / `Cmd+Shift+E` | Capture screen |

### In-app

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Capture screen |
| `Ctrl+Z` | Undo last annotation |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+,` | Open settings |
| `Ctrl+0` | Reset canvas zoom |
| `Delete` / `Backspace` | Remove selected annotation |
| `V` | Select tool |
| `C` | Circle tool |
| `S` | Rectangle tool |
| `A` | Arrow tool |
| `F` | Freehand tool |
| `T` | Text tool |

### Canvas navigation

| Action | How |
|--------|-----|
| Zoom | Scroll wheel |
| Pan | Middle-click drag |
| Reset view | `Ctrl+0` |

---

## How It Works

```
Ctrl+Shift+E
     │
     ▼
Window hides → Full-screen capture → Window restores
     │
     ▼
Screenshot loads into canvas
     │
     ▼
Annotate with tools → Add notes → Set severity
     │
     ▼
Prompt generated (markdown with coords + git context)
     │
     ├── Copy to clipboard
     ├── Drag to AI chat
     └── MCP server (AI reads automatically)
```

---

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) ≥ 18
- [pnpm](https://pnpm.io/)
- Linux: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`

### Development

```bash
git clone https://github.com/cranknet/CodeEye
cd CodeEye
pnpm install
pnpm tauri dev
```

### Tests

```bash
pnpm test              # TypeScript tests (Vitest)
cd src-tauri && cargo test   # Rust tests
```

### Production build

```bash
pnpm tauri build
```

Output is in `src-tauri/target/release/bundle/`.

---

## Contributing

1. Fork the repo and create a branch
2. Make your changes — run `pnpm dlx ultracite fix` before committing
3. Ensure all tests pass (`pnpm test` + `cargo test`)
4. Open a pull request

See [`docs/plans/`](docs/plans/) for the implementation roadmap.

---

## Privacy

CodeEye is fully local:

- No telemetry, no analytics, no crash reporting
- No network calls except optional auto-update checks (GitHub Releases API)
- All data stored at `~/.codeeye/` — delete the folder to remove everything
- MCP server communicates over stdio only (local process)

---

## License

MIT — see [LICENSE](LICENSE).
