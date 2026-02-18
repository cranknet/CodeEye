<script lang="ts">
  import type { createAnnotationStore } from "$lib/state/annotations.svelte";
  import type { createToolStore, ToolType } from "$lib/state/tools.svelte";

  interface Props {
    annotationState: ReturnType<typeof createAnnotationStore>;
    toolState: ReturnType<typeof createToolStore>;
  }

  let { toolState, annotationState }: Props = $props();

  const tools: {
    type: ToolType;
    label: string;
    shortcut: string;
    icon: string;
  }[] = [
    { type: "select", label: "Select", shortcut: "V", icon: "◇" },
    { type: "rectangle", label: "Rectangle", shortcut: "S", icon: "□" },
    { type: "circle", label: "Circle", shortcut: "C", icon: "○" },
    { type: "arrow", label: "Arrow", shortcut: "A", icon: "→" },
    { type: "freehand", label: "Freehand", shortcut: "F", icon: "∿" },
    { type: "text", label: "Text", shortcut: "T", icon: "T" },
  ];

  const severities: {
    value: "critical" | "minor" | "suggestion";
    label: string;
    color: string;
  }[] = [
    { value: "critical", label: "Critical", color: "#ef4444" },
    { value: "minor", label: "Minor", color: "#eab308" },
    { value: "suggestion", label: "Suggest", color: "#3b82f6" },
  ];
</script>

<div
  class="flex items-center gap-1 px-3 py-1.5 bg-[#111111] border-b border-white/[0.06] select-none"
>
  <!-- Tool buttons -->
  <div class="flex items-center gap-0.5 border-r border-white/[0.06] pr-2 mr-1">
    {#each tools as tool}
      <button
        type="button"
        class="group relative flex items-center justify-center w-8 h-8 rounded-md text-sm transition-all duration-100
          {toolState.activeTool === tool.type
            ? 'bg-[#f97316]/15 text-[#f97316] ring-1 ring-[#f97316]/30'
            : 'text-white/50 hover:text-white/80 hover:bg-white/[0.04]'}"
        onclick={() => toolState.setTool(tool.type)}
        title="{tool.label} ({tool.shortcut})"
      >
        <span class="text-base leading-none">{tool.icon}</span>
        <!-- Shortcut hint -->
        <span
          class="absolute -bottom-0.5 -right-0.5 text-[8px] font-mono leading-none opacity-0 group-hover:opacity-60 transition-opacity"
        >
          {tool.shortcut}
        </span>
      </button>
    {/each}
  </div>

  <!-- Severity selector -->
  <div class="flex items-center gap-0.5 border-r border-white/[0.06] pr-2 mr-1">
    {#each severities as sev}
      <button
        type="button"
        class="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all duration-100 uppercase tracking-wider
          {toolState.activeSeverity === sev.value
            ? 'ring-1'
            : 'opacity-40 hover:opacity-70'}"
        style:color={sev.color}
        style:ring-color={toolState.activeSeverity === sev.value ? `${sev.color}60` : 'transparent'}
        style:background-color={toolState.activeSeverity === sev.value ? `${sev.color}15` : 'transparent'}
        onclick={() => toolState.setSeverity(sev.value)}
      >
        <span
          class="w-1.5 h-1.5 rounded-full"
          style:background-color={sev.color}
        ></span>
        {sev.label}
      </button>
    {/each}
  </div>

  <!-- Quick labels -->
  <div class="flex items-center gap-1 flex-1 overflow-x-auto scrollbar-hide">
    {#each toolState.quickLabels as label}
      <button
        type="button"
        class="shrink-0 px-2 py-0.5 rounded text-[10px] font-mono tracking-wide transition-all duration-100
          {toolState.activeQuickLabel === label.name
            ? 'bg-[#f97316]/15 text-[#f97316] ring-1 ring-[#f97316]/30'
            : 'text-white/30 hover:text-white/60 bg-white/[0.02] hover:bg-white/[0.04]'}"
        onclick={() => toolState.selectQuickLabel(
          toolState.activeQuickLabel === label.name ? null : label.name
        )}
      >
        {label.name}
      </button>
    {/each}
  </div>

  <!-- Undo/Redo -->
  <div class="flex items-center gap-0.5 border-l border-white/[0.06] pl-2 ml-1">
    <button
      type="button"
      class="w-7 h-7 flex items-center justify-center rounded text-xs transition-colors
        {annotationState.canUndo ? 'text-white/50 hover:text-white/80 hover:bg-white/[0.04]' : 'text-white/15 cursor-not-allowed'}"
      onclick={() => annotationState.undo()}
      disabled={!annotationState.canUndo}
      title="Undo (Ctrl+Z)"
    >
      ↶
    </button>
    <button
      type="button"
      class="w-7 h-7 flex items-center justify-center rounded text-xs transition-colors
        {annotationState.canRedo ? 'text-white/50 hover:text-white/80 hover:bg-white/[0.04]' : 'text-white/15 cursor-not-allowed'}"
      onclick={() => annotationState.redo()}
      disabled={!annotationState.canRedo}
      title="Redo (Ctrl+Shift+Z)"
    >
      ↷
    </button>
  </div>
</div>
