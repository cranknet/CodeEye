<script lang="ts">
  import Canvas from "$lib/components/Canvas.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import { createAnnotationStore } from "$lib/state/annotations.svelte";
  import { createCanvasStore } from "$lib/state/canvas.svelte";
  import { createToolStore } from "$lib/state/tools.svelte";

  // App-level stores — single source of truth
  const canvasState = createCanvasStore();
  const annotationState = createAnnotationStore();
  const toolState = createToolStore();

  // Session metadata
  let pageName = $state("");
  let generalNotes = $state("");

  // Shared selection state (canvas ↔ sidebar)
  let selectedId: string | null = $state(null);

  // Placeholder — will be replaced by actual capture in Task 18-20
  let imageSrc: string | null = $state(null);
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

      <!-- Zoom indicator -->
      <div
        class="absolute bottom-3 left-3 text-[10px] font-mono text-white/20 pointer-events-none"
      >
        {Math.round(canvasState.zoom * 100)}%
      </div>
    </div>

    <!-- Sidebar -->
    <Sidebar
      {annotationState}
      {selectedId}
      onselect={(id) => { selectedId = id; }}
      {pageName}
      {generalNotes}
      onpagechange={(v) => { pageName = v; }}
      onnoteschange={(v) => { generalNotes = v; }}
    />
  </div>
</main>
