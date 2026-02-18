<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Canvas from "$lib/components/Canvas.svelte";
  import ExportBar from "$lib/components/ExportBar.svelte";
  import FirstRun from "$lib/components/FirstRun.svelte";
  import IntegrationHub from "$lib/components/IntegrationHub.svelte";
  import PromptPreview from "$lib/components/PromptPreview.svelte";
  import RegionOverlay from "$lib/components/RegionOverlay.svelte";
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

  // Capture state
  let imageSrc: string | null = $state(null);
  let showOverlay = $state(false);
  let isCapturing = $state(false);

  // Modal states
  let showPromptPreview = $state(false);
  let showSessionList = $state(false);
  let showSettings = $state(false);
  let showIntegrations = $state(false);
  let showFirstRun = $state(false);

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

  /** Start the capture flow: show the region overlay */
  function startCapture() {
    showOverlay = true;
  }

  /** Handle region selection confirmation */
  async function handleRegionConfirm(region: {
    x: number;
    y: number;
    width: number;
    height: number;
  }) {
    showOverlay = false;
    isCapturing = true;

    try {
      const base64: string = await invoke("capture_region", {
        monitorId: 0,
        x: Math.round(region.x),
        y: Math.round(region.y),
        width: Math.round(region.width),
        height: Math.round(region.height),
      });

      imageSrc = `data:image/png;base64,${base64}`;
      annotationState.clear();
    } catch (err) {
      console.error("Capture failed:", err);
    } finally {
      isCapturing = false;
    }
  }

  function handleOverlayCancel() {
    showOverlay = false;
  }

  /** Quick capture: full primary monitor */
  async function captureFullScreen() {
    isCapturing = true;
    try {
      const base64: string = await invoke("capture_screen", { monitorId: 0 });
      imageSrc = `data:image/png;base64,${base64}`;
      annotationState.clear();
    } catch (err) {
      console.error("Full screen capture failed:", err);
    } finally {
      isCapturing = false;
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

  /** Handle first-run completion — persist to config */
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

  /** Load an image from a data URL or blob URL */
  function loadImage(src: string) {
    imageSrc = src;
    annotationState.clear();
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

  let isDragOver = $state(false);

  // Register keyboard shortcuts
  shortcuts.register({
    key: "n",
    ctrl: true,
    description: "New capture",
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

  // Check first-run config and load sessions on mount
  async function initApp() {
    try {
      const config = await invoke<{ first_run_complete: boolean }>(
        "load_app_config"
      );
      if (!config.first_run_complete) {
        showFirstRun = true;
      }
    } catch {
      // Config not found or unreadable — show first run
      showFirstRun = true;
    }
    sessionState.loadSessions();
  }

  initApp();
</script>

<svelte:window onpaste={handlePaste} onkeydown={shortcuts.handleKeyDown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<main
  class="h-screen w-screen bg-[#0a0a0a] text-[#fafafa] flex flex-col overflow-hidden
    {isDragOver ? 'ring-2 ring-inset ring-[#f97316]/40' : ''}"
  ondrop={handleDrop}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
>
  <!-- Toolbar -->
  <Toolbar {toolState} {annotationState} />

  <!-- Editor area: Canvas + Sidebar -->
  <div class="flex flex-1 min-h-0">
    <!-- Canvas (fills remaining space) -->
    <div class="flex-1 relative min-w-0">
      <Canvas
        {canvasState}
        {annotationState}
        {toolState}
        {imageSrc}
        bind:selectedId
      />

      <!-- Zoom indicator -->
      <div
        class="absolute bottom-3 left-3 flex items-center gap-2 text-[10px] font-mono text-white/20 pointer-events-none"
      >
        {Math.round(canvasState.zoom * 100)}%
      </div>

      <!-- Capture button (when no image loaded) -->
      {#if !imageSrc && !isCapturing}
        <div
          class="absolute inset-0 flex items-center justify-center pointer-events-none"
        >
          <div class="flex flex-col items-center gap-3 pointer-events-auto">
            <p class="text-sm text-white/30 font-mono">No screenshot loaded</p>
            <div class="flex gap-2">
              <button
                type="button"
                class="px-4 py-2 text-xs font-medium bg-[#f97316] text-[#0a0a0a] rounded-md
                  hover:bg-[#f97316]/90 transition-colors"
                onclick={startCapture}
              >
                Select Region
              </button>
              <button
                type="button"
                class="px-4 py-2 text-xs font-medium bg-white/[0.06] text-white/60 rounded-md
                  hover:bg-white/[0.1] transition-colors"
                onclick={captureFullScreen}
              >
                Full Screen
              </button>
            </div>
            <p class="text-[10px] text-white/15 font-mono">
              Or drag & drop an image · Ctrl+V to paste
            </p>
          </div>
        </div>
      {/if}

      {#if isCapturing}
        <div
          class="absolute inset-0 flex items-center justify-center bg-black/30"
        >
          <p class="text-sm text-white/50 font-mono animate-pulse">
            Capturing...
          </p>
        </div>
      {/if}
    </div>

    <!-- Sidebar -->
    <Sidebar
      {annotationState}
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

  <!-- Export bar (bottom) -->
  {#if imageSrc}
    <ExportBar
      {promptMarkdown}
      onshowpreview={() => {
        showPromptPreview = true;
      }}
    />
  {/if}
</main>

<!-- Prompt preview modal -->
{#if showPromptPreview}
  <PromptPreview
    {promptMarkdown}
    onclose={() => {
      showPromptPreview = false;
    }}
  />
{/if}

<!-- Session list modal -->
{#if showSessionList}
  <SessionList
    {sessionState}
    onrestore={handleSessionRestore}
    onclose={() => {
      showSessionList = false;
    }}
  />
{/if}

<!-- Settings modal -->
{#if showSettings}
  <Settings
    onclose={() => {
      showSettings = false;
    }}
    onshowintegrations={() => {
      showSettings = false;
      showIntegrations = true;
    }}
  />
{/if}

<!-- Integration hub modal -->
{#if showIntegrations}
  <IntegrationHub
    onclose={() => {
      showIntegrations = false;
    }}
  />
{/if}

<!-- First-run experience -->
{#if showFirstRun}
  <FirstRun oncomplete={handleFirstRunComplete} />
{/if}

<!-- Region overlay (fullscreen, above everything) -->
{#if showOverlay}
  <RegionOverlay
    onconfirm={handleRegionConfirm}
    oncancel={handleOverlayCancel}
  />
{/if}

<!-- Toast notifications -->
<Toast {toastState} />
