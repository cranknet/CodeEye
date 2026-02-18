# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CodeEye — "The fastest way to report UI bugs to AI." A Tauri v2 desktop app that lets developers capture screenshots, annotate UI issues, and send structured feedback to AI coding assistants via MCP (Model Context Protocol).

**Core principle:** Every feature must reduce time-to-feedback. If it adds a step, it doesn't ship.

**Status:** Pre-development. PRD finalized at `codeeye-prd.md`, implementation plan at `docs/plans/2026-02-18-codeeye-v1-implementation.md`.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Runtime | Tauri v2 (Rust backend, ~8MB binary) |
| Frontend | Svelte 5 + Vite + Tailwind CSS |
| Canvas | Canvas 2D API, single element, requestAnimationFrame loop |
| State | Svelte 5 Runes (`$state`, `$derived`, `$effect`); pure TS for geometry utils |
| MCP | Embedded stdio server (runs inside Tauri, not sidecar) |
| Storage | File-based at `~/.codeeye/` (JSON + PNG, no database) |
| i18n | Paraglide JS (or svelte-i18n), English only at launch |
| Linting | Ultracite (Svelte/TS), cargo fmt + clippy (Rust) |
| Testing | Vitest (TS), cargo test (Rust), Tauri WebDriver (E2E) |

## Commands

```bash
# Development
npm run tauri dev          # Run Tauri app in dev mode (hot-reload)

# Frontend
npm run dev                # Vite dev server only (no Tauri)
npm run build              # Vite production build

# Testing
npm run test               # vitest run (single run)
npm run test:watch         # vitest (watch mode)
cargo test                 # Rust tests (run from src-tauri/)

# Linting
npm run lint               # ultracite check
npm run lint:fix           # ultracite --fix
cargo fmt --check          # Rust formatting check (from src-tauri/)
cargo clippy               # Rust linting (from src-tauri/)

# Build
npm run tauri build        # Production build (platform-specific installer)
```

## Architecture

### IPC Boundary — What Lives Where

**Rust backend (`src-tauri/`):**
- Screen capture + multi-monitor detection (xcap crate)
- Global hotkey registration (`Cmd+Shift+E` / `Ctrl+Shift+E`)
- File I/O (sessions, images, config at `~/.codeeye/`)
- Git context detection (project, branch, recent diff)
- MCP server + adapter management
- Image compression/resizing pipeline
- Auto-cleaner (time-based 30d or count-based 200 sessions)
- System tray lifecycle, single instance enforcement

**Svelte frontend (`src/`):**
- Canvas rendering + annotation engine (circle, rectangle, arrow, freehand, text)
- All UI state via Svelte 5 runes
- Undo/redo stack (state snapshots)
- Prompt generation (markdown assembly)
- Drag & drop, keyboard shortcuts for tools
- Prompt preview & editing

### Canvas Rendering Pipeline

```
Screenshot (base image) → Transform (zoom/pan) → Annotation layer → Selection handles → Cursor overlay
```

Single `<canvas>`, one `requestAnimationFrame` render loop. Pure TypeScript geometry utilities — no Svelte state in math functions.

### Storage Layout

```
~/.codeeye/
├── config.json
├── sessions/
│   ├── index.json              # Lightweight manifest
│   └── <session-id>/
│       ├── meta.json           # Git context, annotations, notes, tags
│       ├── original.png        # Full resolution
│       ├── compressed.png      # Resized for export
│       └── annotated.png       # Rendered with annotations
└── logs/                       # Max 5 files, 5MB each
```

### Key Data Types

Annotations have `type` (circle | rectangle | arrow | freehand | text), sequential `number` (#1, #2...), `bounds`, optional `points`, `label`, `comment`, and `severity` (critical | minor | suggestion).

Sessions track `status` (open | resolved), `frames`, `annotations`, `git` context (project, branch, working_directory, suggested_file, recent_diff), and `viewport`.

### MCP Integration

The MCP server is embedded (stdio), not a sidecar process. For v1, it's **read-only** — AI tools can read feedback but not trigger captures.

Three tools exposed: `get_ui_feedback`, `list_ui_feedback`, `resolve_ui_feedback`.

### Existing Proof-of-Concept

`ui-feedback-mcp/` contains a working dual-protocol MCP server (HTTP + stdio) used for prototyping. It stores feedback to `.feedback/` directory. This will be superseded by the embedded Tauri MCP server.

## Key Conventions

- **Svelte 5 runes everywhere** — use `$state`, `$derived`, `$effect` for all reactive state. No legacy Svelte stores.
- **Pure TS for geometry** — canvas math (hit-testing, transforms, intersections) must be pure functions with no Svelte/framework dependencies.
- **Percentage + absolute coordinates** — annotations use percentage positions for responsive context, absolute pixels for precision.
- **Brand color:** `#f97316` (orange) for primary/accent. Severity colors: critical `#ef4444`, minor `#eab308`, suggestion `#3b82f6`.
- **Error handling:** Never crash. Recoverable errors show a toast and continue. Fatal errors (permissions, storage) show an error screen with guidance.
- **Privacy:** Zero telemetry. No network calls except auto-update checks (GitHub Releases API) and local MCP stdio.
- **License:** MIT

## Implementation Plan

The implementation plan at `docs/plans/2026-02-18-codeeye-v1-implementation.md` defines 10 phases (44 tasks) with specific files, steps, and commit messages for each. Follow it task-by-task using `superpowers:executing-plans`.
