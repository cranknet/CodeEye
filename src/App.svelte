<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Canvas from "$lib/components/Canvas.svelte";
  import ExportBar from "$lib/components/ExportBar.svelte";
  import FeedbackLoop from "$lib/components/FeedbackLoop.svelte";
  import FirstRun from "$lib/components/FirstRun.svelte";
  import IntegrationHub from "$lib/components/IntegrationHub.svelte";
  import PromptPreview from "$lib/components/PromptPreview.svelte";

  import SessionList from "$lib/components/SessionList.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import { createAnnotationStore } from "$lib/state/annotations.svelte";
  import { createCanvasStore } from "$lib/state/canvas.svelte";
  import { createSessionStore } from "$lib/state/sessions.svelte";
  import { createToastStore } from "$lib/state/toast.svelte";
  import { createToolStore } from "$lib/state/tools.svelte";
  import { generatePrompt } from "$lib/utils/export";
  import { createShortcutRegistry } from "$lib/utils/shortcuts";

  // App-level stores — single source of truth
  const canvasState = createCanvasStore();
  const annotationState = createAnnotationStore();
  const toolState = createToolStore();
  const sessionState = createSessionStore();
  const toastState = createToastStore();
  const shortcuts = createShortcutRegistry();

  // Session metadata
  let pageName = $state("");
  let generalNotes = $state("");

  // Shared selection state (canvas ↔ sidebar)
  let selectedId: string | null = $state(null);

  // Theme — light by default, loaded from config on init
  let theme = $state("light");

  // Track OS dark mode preference so "system" theme updates live
  let osDark = $state(
    window.matchMedia("(prefers-color-scheme: dark)").matches
  );

  $effect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      osDark = e.matches;
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  });

  // Resolved theme: "system" maps to actual "dark"/"light" based on OS preference
  let resolvedTheme = $derived.by(() => {
    if (theme === "system") {
      return osDark ? "dark" : "light";
    }
    return theme;
  });

  // Apply theme attribute to <html> so all CSS variables update globally
  $effect(() => {
    document.documentElement.setAttribute("data-theme", resolvedTheme);
  });

  // Canvas background derived from resolved theme
  let canvasBg = $derived(resolvedTheme === "dark" ? "#1a1a1a" : "#e8e8e8");

  // Derive current image from frame list
  let imageSrc = $derived(canvasState.currentImageSrc);

  // Capture state
  let isCapturing = $state(false);
  let isDragOver = $state(false);

  // Modal states
  let showPromptPreview = $state(false);
  let showSessionList = $state(false);
  let showSettings = $state(false);
  let showIntegrations = $state(false);
  let showFirstRun = $state(false);
  let showFeedbackLoop = $state(false);

  // Derived prompt markdown
  let promptMarkdown = $derived(
    generatePrompt({
      pageName,
      generalNotes,
      annotations: annotationState.annotations,
      viewport:
        canvasState.imageWidth > 0
          ? { width: canvasState.imageWidth, height: canvasState.imageHeight }
          : undefined,
    })
  );

  /**
   * Capture the full primary monitor.
   * Hides the window first so the app UI doesn't appear in the screenshot.
   */
  async function startCapture() {
    if (isCapturing) {
      return;
    }
    isCapturing = true;
    const win = getCurrentWindow();
    try {
      await win.hide();
      // Give the OS time to remove the window from the compositor
      await new Promise<void>((resolve) => setTimeout(resolve, 200));
      const base64: string = await invoke("capture_screen", { monitorId: 0 });
      const dataUrl = `data:image/png;base64,${base64}`;
      if (canvasState.frameCount > 0) {
        addFrame(dataUrl);
      } else {
        loadImage(dataUrl);
      }
    } catch (err) {
      console.error("Capture failed:", err);
      toastState.error("Screenshot capture failed");
    } finally {
      try {
        await win.show();
      } catch {
        /* window may already be visible */
      }
      isCapturing = false;
    }
  }

  /** Open an image from the file manager using Tauri dialog */
  async function openFromFile() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { readFile } = await import("@tauri-apps/plugin-fs");
      const path = await open({
        filters: [
          {
            name: "Images",
            extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"],
          },
        ],
      });
      if (!path) {
        return;
      }
      const bytes = await readFile(path as string);
      const binary = Array.from(new Uint8Array(bytes))
        .map((b) => String.fromCharCode(b))
        .join("");
      const base64 = btoa(binary);
      loadImage(`data:image/png;base64,${base64}`);
    } catch (err) {
      console.error("File open failed:", err);
      toastState.error("Failed to open image file");
    }
  }

  /** Restore a session by ID */
  async function handleSessionRestore(sessionId: string) {
    showSessionList = false;
    sessionState.setCurrentSession(sessionId);
    const session = await sessionState.loadSession(sessionId);
    if (session) {
      pageName = session.pageName;
      generalNotes = session.generalNotes;
      annotationState.clear();
      for (const a of session.annotations) {
        annotationState.add(a);
      }
      toastState.success(`Loaded "${session.pageName || "session"}"`);
    } else {
      toastState.error("Failed to load session");
    }
  }

  /** Handle first-run completion */
  async function handleFirstRunComplete() {
    showFirstRun = false;
    try {
      const config = await invoke<Record<string, unknown>>("load_app_config");
      config.first_run_complete = true;
      await invoke("save_app_config", { config });
    } catch (err) {
      console.error("Failed to save first-run state:", err);
    }
  }

  /** Load an image as a new single-frame session */
  function loadImage(src: string) {
    canvasState.setFrames([src]);
    annotationState.clear();
  }

  /** Add an image as an additional frame (batch mode) */
  function addFrame(src: string) {
    canvasState.addFrame(src);
  }

  /** Handle file drop */
  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    const file = e.dataTransfer?.files[0];
    if (file?.type.startsWith("image/")) {
      const reader = new FileReader();
      reader.onload = () => {
        loadImage(reader.result as string);
      };
      reader.readAsDataURL(file);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave() {
    isDragOver = false;
  }

  /** Handle clipboard paste */
  function handlePaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) {
      return;
    }
    for (const item of items) {
      if (item.type.startsWith("image/")) {
        const blob = item.getAsFile();
        if (blob) {
          const reader = new FileReader();
          reader.onload = () => {
            loadImage(reader.result as string);
          };
          reader.readAsDataURL(blob);
        }
        break;
      }
    }
  }

  /** Handle theme change from Settings */
  function handleThemeChange(newTheme: string) {
    theme = newTheme;
  }

  // Register keyboard shortcuts
  shortcuts.register({
    key: "n",
    ctrl: true,
    description: "Capture screen",
    action: startCapture,
  });
  shortcuts.register({
    key: "z",
    ctrl: true,
    description: "Undo",
    action: () => annotationState.undo(),
  });
  shortcuts.register({
    key: "z",
    ctrl: true,
    shift: true,
    description: "Redo",
    action: () => annotationState.redo(),
  });
  shortcuts.register({
    key: ",",
    ctrl: true,
    description: "Open settings",
    action: () => {
      showSettings = true;
    },
  });

  // Check first-run state + load sessions on mount
  async function initApp() {
    try {
      const config = await invoke<{
        first_run_complete: boolean;
        theme: string;
      }>("load_app_config");
      if (!config.first_run_complete) {
        showFirstRun = true;
      }
      if (config.theme) {
        theme = config.theme;
      }
    } catch {
      // Config not found or unreadable — show first run
      showFirstRun = true;
    }
    sessionState.loadSessions();
  }

  // Listen for Tauri backend events
  $effect(() => {
    const unlistenHotkey = listen("capture-hotkey", () => startCapture());
    const unlistenTray = listen("tray-capture", () => startCapture());
    const unlistenConflict = listen<string>("hotkey-conflict", (event) => {
      toastState.error(
        `Hotkey unavailable: ${event.payload}. Use the tray menu instead.`
      );
    });
    return () => {
      unlistenHotkey.then((fn) => fn());
      unlistenTray.then((fn) => fn());
      unlistenConflict.then((fn) => fn());
    };
  });

  initApp();
</script>

<svelte:window onpaste={handlePaste} onkeydown={shortcuts.handleKeyDown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<main
  class="h-screen w-screen flex flex-col overflow-hidden bg-[var(--bg-app)] text-[var(--text-primary)]
    {isDragOver ? 'ring-2 ring-inset ring-[var(--accent)]/50' : ''}"
  ondrop={handleDrop}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
>
  <!-- Header row: Toolbar + nav actions -->
  <div
    class="flex items-stretch border-b border-[var(--border)] bg-[var(--bg-surface)] shrink-0"
    style="box-shadow: var(--shadow-xs);"
  >
    <Toolbar {toolState} {annotationState} />

    <!-- Right-side nav actions -->
    <div
      class="flex items-center gap-0.5 px-2 border-l border-[var(--border)] shrink-0"
    >
      <button
        type="button"
        class="px-2.5 py-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-secondary)]
          hover:bg-[var(--bg-surface-hover)] rounded-md transition-colors"
        onclick={() => {
          showSessionList = true;
        }}
        title="Session history"
      >
        History
      </button>
      <button
        type="button"
        class="w-8 h-8 flex items-center justify-center text-sm text-[var(--text-muted)]
          hover:text-[var(--text-secondary)] hover:bg-[var(--bg-surface-hover)] rounded-md transition-colors"
        onclick={() => {
          showSettings = true;
        }}
        title="Settings (Ctrl+,)"
      >
        ⚙
      </button>
    </div>
  </div>

  <!-- Frame strip (only when multiple frames) -->
  {#if canvasState.frameCount > 1}
    <div
      class="flex items-center gap-1 px-3 py-1.5 bg-[var(--bg-sunken)] border-b border-[var(--border)] shrink-0 overflow-x-auto"
    >
      {#each canvasState.frames as _src, i}
        <button
          type="button"
          class="px-3 py-1 text-xs font-medium rounded-md transition-colors shrink-0
            {canvasState.currentFrame === i
              ? 'bg-[var(--accent)] text-[var(--text-on-accent)]'
              : 'bg-[var(--bg-surface)] text-[var(--text-muted)] hover:text-[var(--text-secondary)] border border-[var(--border)]'}"
          onclick={() => canvasState.setFrame(i)}
        >
          Frame {i + 1}
        </button>
      {/each}
      <button
        type="button"
        class="px-2 py-1 text-xs text-[var(--text-faint)] hover:text-[var(--text-secondary)]
          hover:bg-[var(--bg-surface-hover)] rounded-md transition-colors shrink-0"
        onclick={startCapture}
        title="Add another frame"
      >
        + Add
      </button>

      <div class="ml-auto shrink-0">
        <button
          type="button"
          class="px-2.5 py-1 text-xs font-medium text-[var(--accent)] hover:bg-[var(--accent-subtle)]
            rounded-md transition-colors"
          onclick={() => { showFeedbackLoop = true; }}
          title="Compare first and last frames"
        >
          Compare
        </button>
      </div>
    </div>
  {/if}

  <!-- Editor area: Canvas + Sidebar -->
  <div class="flex flex-1 min-h-0">
    <!-- Canvas (fills remaining space) -->
    <div class="flex-1 relative min-w-0">
      <Canvas
        {canvasState}
        {annotationState}
        {toolState}
        {imageSrc}
        {canvasBg}
        bind:selectedId
      />

      <!-- Zoom indicator -->
      <div
        class="absolute bottom-3 left-3 text-[11px] font-mono text-[var(--text-faint)] pointer-events-none select-none
          bg-[var(--bg-surface)]/80 px-1.5 py-0.5 rounded backdrop-blur-sm"
      >
        {Math.round(canvasState.zoom * 100)}%
      </div>

      <!-- Empty state — no screenshot loaded -->
      {#if !imageSrc && !isCapturing}
        <div
          class="absolute inset-0 flex items-center justify-center pointer-events-none"
        >
          <div class="flex flex-col items-center gap-5 pointer-events-auto">
            <!-- Placeholder icon -->
            <div
              class="w-14 h-14 rounded-2xl bg-[var(--bg-surface)] border border-[var(--border)]
                flex items-center justify-center text-3xl select-none"
              style="box-shadow: var(--shadow-md);"
            >
              📸
            </div>

            <div class="text-center space-y-1">
              <p class="text-sm font-medium text-[var(--text-secondary)]">
                No screenshot loaded
              </p>
              <p class="text-xs text-[var(--text-muted)]">
                Capture your screen or drop an image here
              </p>
            </div>

            <div class="flex gap-2">
              <button
                type="button"
                class="px-4 py-2 text-sm font-medium bg-[var(--accent)] text-[var(--text-on-accent)]
                  rounded-lg hover:bg-[var(--accent-hover)] transition-colors"
                style="box-shadow: var(--shadow-sm);"
                onclick={startCapture}
              >
                Capture Screen
              </button>
              <button
                type="button"
                class="px-4 py-2 text-sm font-medium bg-[var(--bg-surface)] text-[var(--text-secondary)]
                  border border-[var(--border)] rounded-lg hover:bg-[var(--bg-surface-hover)] transition-colors"
                style="box-shadow: var(--shadow-sm);"
                onclick={openFromFile}
              >
                Browse File
              </button>
            </div>

            <p class="text-xs text-[var(--text-faint)] font-mono">
              Or drag & drop · Ctrl+V to paste
            </p>
          </div>
        </div>
      {/if}

      <!-- Capturing spinner -->
      {#if isCapturing}
        <div
          class="absolute inset-0 flex items-center justify-center"
          style="background: var(--bg-overlay);"
        >
          <div
            class="flex items-center gap-2.5 px-4 py-2.5 bg-[var(--bg-surface)]
              rounded-lg border border-[var(--border)]"
            style="box-shadow: var(--shadow-lg);"
          >
            <span class="text-sm text-[var(--text-secondary)] animate-pulse"
              >Capturing…</span
            >
          </div>
        </div>
      {/if}
    </div>

    <!-- Sidebar -->
    <Sidebar
      {annotationState}
      {canvasState}
      {selectedId}
      onselect={(id) => {
        selectedId = id;
      }}
      {pageName}
      {generalNotes}
      onpagechange={(v) => {
        pageName = v;
      }}
      onnoteschange={(v) => {
        generalNotes = v;
      }}
    />
  </div>

  <!-- Export bar (bottom, only when screenshot loaded) -->
  {#if imageSrc}
    <ExportBar
      {promptMarkdown}
      onshowpreview={() => {
        showPromptPreview = true;
      }}
    />
  {/if}
</main>

<!-- ── Modals ──────────────────────────────── -->

{#if showPromptPreview}
  <PromptPreview
    {promptMarkdown}
    onclose={() => {
      showPromptPreview = false;
    }}
  />
{/if}

{#if showSessionList}
  <SessionList
    {sessionState}
    onrestore={handleSessionRestore}
    onclose={() => {
      showSessionList = false;
    }}
  />
{/if}

{#if showSettings}
  <Settings
    onclose={() => {
      showSettings = false;
    }}
    onshowintegrations={() => {
      showSettings = false;
      showIntegrations = true;
    }}
    onthemechange={handleThemeChange}
  />
{/if}

{#if showIntegrations}
  <IntegrationHub
    onclose={() => {
      showIntegrations = false;
    }}
  />
{/if}

{#if showFirstRun}
  <FirstRun oncomplete={handleFirstRunComplete} />
{/if}

{#if showFeedbackLoop && canvasState.frameCount >= 2}
  <FeedbackLoop
    beforeSrc={canvasState.frames[0]}
    afterSrc={canvasState.frames[canvasState.frameCount - 1]}
    annotations={annotationState.annotations.filter(
      (a) => a.frame === 0 || a.frame === canvasState.frameCount - 1
    )}
    onclose={() => { showFeedbackLoop = false; }}
  />
{/if}

<Toast {toastState} />
