<script lang="ts">
  import { t } from 'svelte-i18n';
  import Canvas from '$lib/components/Canvas.svelte';
  import { createCanvasStore } from '$lib/state/canvas.svelte';
  import { createAnnotationStore } from '$lib/state/annotations.svelte';
  import { createToolStore } from '$lib/state/tools.svelte';

  const canvasState = createCanvasStore();
  const annotationState = createAnnotationStore();
  const toolState = createToolStore();

  // Placeholder — will be replaced by actual capture in Task 18-20
  let imageSrc: string | null = $state(null);
</script>

<main class="h-screen w-screen bg-[#0a0a0a] text-[#fafafa] flex flex-col overflow-hidden">
  <!-- Minimal top bar showing app name + active tool -->
  <header class="flex items-center justify-between px-4 py-2 border-b border-white/10 shrink-0">
    <span class="text-sm font-bold tracking-wide text-[#f97316]">{$t('app.name')}</span>
    <span class="text-xs text-white/40 font-mono uppercase">{toolState.activeTool}</span>
  </header>

  <!-- Canvas fills remaining space -->
  <div class="flex-1 relative min-h-0">
    <Canvas {canvasState} {annotationState} {toolState} {imageSrc} />
  </div>
</main>
