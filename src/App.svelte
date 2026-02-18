<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Canvas from "$lib/components/Canvas.svelte";
  import ExportBar from "$lib/components/ExportBar.svelte";
  import PromptPreview from "$lib/components/PromptPreview.svelte";
  import RegionOverlay from "$lib/components/RegionOverlay.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import { createAnnotationStore } from "$lib/state/annotations.svelte";
  import { createCanvasStore } from "$lib/state/canvas.svelte";
  import { createToolStore } from "$lib/state/tools.svelte";
  import { generatePrompt } from "$lib/utils/export";

  // App-level stores — single source of truth
  const canvasState = createCanvasStore();
  const annotationState = createAnnotationStore();
  const toolState = createToolStore();

  // Session metadata
  let pageName = $state("");
  let generalNotes = $state("");

  // Shared selection state (canvas ↔ sidebar)
  let selectedId: string | null = $state(null);

  // Capture state
  let imageSrc: string | null = $state(null);
  let showOverlay = $state(false);
  let isCapturing = $state(false);

  // Prompt preview
  let showPromptPreview = $state(false);

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
      // Capture region from primary monitor (monitor 0)
      const base64: string = await invoke("capture_region", {
        monitorId: 0,
        x: Math.round(region.x),
        y: Math.round(region.y),
        width: Math.round(region.width),
        height: Math.round(region.height),
      });

      imageSrc = `data:image/png;base64,${base64}`;

      // Clear previous annotations for new capture
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
</script>

<main
  class="h-screen w-screen bg-[#0a0a0a] text-[#fafafa] flex flex-col overflow-hidden"
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

      <!-- Zoom indicator + capture button -->
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
      onshowpreview={() => { showPromptPreview = true; }}
    />
  {/if}
</main>

<!-- Prompt preview modal -->
{#if showPromptPreview}
  <PromptPreview
    {promptMarkdown}
    onclose={() => { showPromptPreview = false; }}
  />
{/if}

<!-- Region overlay (fullscreen, above everything) -->
{#if showOverlay}
  <RegionOverlay
    onconfirm={handleRegionConfirm}
    oncancel={handleOverlayCancel}
  />
{/if}
