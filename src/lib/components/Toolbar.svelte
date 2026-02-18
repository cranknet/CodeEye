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

<!-- Fills flex-1 of the App.svelte header row -->
<div
  class="flex flex-1 items-center gap-1 px-3 py-2 select-none overflow-hidden min-w-0"
>
  <!-- Tool buttons -->
  <div
    class="flex items-center gap-0.5 border-r border-[var(--border)] pr-3 mr-1 shrink-0"
  >
    {#each tools as tool}
      <button
        type="button"
        class="group relative flex items-center justify-center w-9 h-9 rounded-md text-base
          transition-all duration-100
          {toolState.activeTool === tool.type
            ? 'bg-[var(--accent-subtle)] text-[var(--accent)]'
            : 'text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)]'}"
        style:box-shadow={toolState.activeTool === tool.type
          ? "inset 0 0 0 1.5px var(--accent)"
          : "none"}
        onclick={() => toolState.setTool(tool.type)}
        title="{tool.label} ({tool.shortcut})"
      >
        <span class="leading-none">{tool.icon}</span>
        <span
          class="absolute -bottom-0.5 -right-0.5 text-[9px] font-mono leading-none
            opacity-0 group-hover:opacity-50 transition-opacity text-[var(--text-muted)]"
        >
          {tool.shortcut}
        </span>
      </button>
    {/each}
  </div>

  <!-- Severity selector -->
  <div
    class="flex items-center gap-0.5 border-r border-[var(--border)] pr-3 mr-1 shrink-0"
  >
    {#each severities as sev}
      <button
        type="button"
        class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-medium
          transition-all duration-100 tracking-wide"
        style:color={sev.color}
        style:background-color={toolState.activeSeverity === sev.value
          ? `${sev.color}18`
          : "transparent"}
        style:box-shadow={toolState.activeSeverity === sev.value
          ? `inset 0 0 0 1.5px ${sev.color}80`
          : "none"}
        style:opacity={toolState.activeSeverity === sev.value ? "1" : "0.45"}
        onclick={() => toolState.setSeverity(sev.value)}
      >
        <span
          class="w-2 h-2 rounded-full shrink-0"
          style:background-color={sev.color}
        ></span>
        {sev.label}
      </button>
    {/each}
  </div>

  <!-- Quick labels (scrollable) -->
  <div class="flex items-center gap-1 flex-1 overflow-x-auto min-w-0">
    {#each toolState.quickLabels as label}
      <button
        type="button"
        class="shrink-0 px-2 py-1 rounded-md text-xs font-mono transition-all duration-100
          {toolState.activeQuickLabel === label.name
            ? 'bg-[var(--accent-subtle)] text-[var(--accent)]'
            : 'text-[var(--text-muted)] hover:text-[var(--text-secondary)] bg-[var(--bg-sunken)] hover:bg-[var(--bg-surface-hover)]'}"
        style:box-shadow={toolState.activeQuickLabel === label.name
          ? "inset 0 0 0 1.5px var(--accent)"
          : "none"}
        onclick={() =>
          toolState.selectQuickLabel(
            toolState.activeQuickLabel === label.name ? null : label.name
          )}
      >
        {label.name}
      </button>
    {/each}
  </div>

  <!-- Undo / Redo -->
  <div
    class="flex items-center gap-0.5 border-l border-[var(--border)] pl-3 ml-1 shrink-0"
  >
    <button
      type="button"
      class="w-8 h-8 flex items-center justify-center rounded-md text-base transition-colors
        {annotationState.canUndo
          ? 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)]'
          : 'text-[var(--text-faint)] cursor-not-allowed'}"
      onclick={() => annotationState.undo()}
      disabled={!annotationState.canUndo}
      title="Undo (Ctrl+Z)"
    >
      ↶
    </button>
    <button
      type="button"
      class="w-8 h-8 flex items-center justify-center rounded-md text-base transition-colors
        {annotationState.canRedo
          ? 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)]'
          : 'text-[var(--text-faint)] cursor-not-allowed'}"
      onclick={() => annotationState.redo()}
      disabled={!annotationState.canRedo}
      title="Redo (Ctrl+Shift+Z)"
    >
      ↷
    </button>
  </div>
</div>
