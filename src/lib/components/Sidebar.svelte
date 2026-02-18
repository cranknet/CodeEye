<script lang="ts">
  import type {
    Annotation,
    createAnnotationStore,
  } from "$lib/state/annotations.svelte";
  import type { createCanvasStore } from "$lib/state/canvas.svelte";
  import CommentPopup from "./CommentPopup.svelte";

  interface Props {
    annotationState: ReturnType<typeof createAnnotationStore>;
    canvasState: ReturnType<typeof createCanvasStore>;
    generalNotes: string;
    onnoteschange: (notes: string) => void;
    onpagechange: (name: string) => void;
    onselect: (id: string | null) => void;
    pageName: string;
    selectedId: string | null;
  }

  let {
    annotationState,
    canvasState,
    selectedId,
    onselect,
    pageName,
    generalNotes,
    onpagechange,
    onnoteschange,
  }: Props = $props();

  // Filter annotations by current frame
  let visibleAnnotations = $derived(
    canvasState.frameCount > 1
      ? annotationState.annotations.filter(
          (a) => a.frame === canvasState.currentFrame
        )
      : annotationState.annotations
  );

  let editingAnnotation: Annotation | null = $state(null);

  const SEVERITY_COLORS: Record<string, string> = {
    critical: "#ef4444",
    minor: "#eab308",
    suggestion: "#3b82f6",
  };

  const SEVERITY_LABELS: Record<string, string> = {
    critical: "Critical",
    minor: "Minor",
    suggestion: "Suggest",
  };

  function handleAnnotationClick(ann: Annotation) {
    onselect(ann.id);
  }

  function handleDoubleClick(ann: Annotation) {
    editingAnnotation = ann;
  }

  function handleDelete(e: MouseEvent, id: string) {
    e.stopPropagation();
    annotationState.remove(id);
    if (selectedId === id) {
      onselect(null);
    }
  }
</script>

<aside
  class="w-72 bg-[var(--bg-surface)] border-l border-[var(--border)] flex flex-col h-full select-none overflow-hidden shrink-0"
>
  <!-- Page info -->
  <div class="p-3 space-y-3 border-b border-[var(--border)] shrink-0">
    <div class="space-y-1.5">
      <label
        class="block text-[11px] font-medium uppercase tracking-wider text-[var(--text-muted)]"
        for="page-name"
      >
        Page name
      </label>
      <input
        id="page-name"
        type="text"
        value={pageName}
        oninput={(e) => onpagechange((e.target as HTMLInputElement).value)}
        placeholder="e.g. /dashboard, LoginForm"
        class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-2.5 py-1.5
          text-sm text-[var(--text-primary)] placeholder:text-[var(--text-faint)]
          focus:outline-none focus:border-[var(--border-focus)] transition-colors"
      >
    </div>

    <div class="space-y-1.5">
      <label
        class="block text-[11px] font-medium uppercase tracking-wider text-[var(--text-muted)]"
        for="general-notes"
      >
        Notes
      </label>
      <textarea
        id="general-notes"
        value={generalNotes}
        oninput={(e) => onnoteschange((e.target as HTMLTextAreaElement).value)}
        placeholder="General notes about this page…"
        rows="2"
        class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-2.5 py-1.5
          text-sm text-[var(--text-primary)] placeholder:text-[var(--text-faint)] resize-none
          focus:outline-none focus:border-[var(--border-focus)] transition-colors"
      ></textarea>
    </div>
  </div>

  <!-- Annotation list header -->
  <div class="flex items-center justify-between px-3 pt-3 pb-2 shrink-0">
    <span
      class="text-[11px] font-medium uppercase tracking-wider text-[var(--text-muted)]"
    >
      Annotations
    </span>
    <span class="text-xs font-mono text-[var(--text-faint)]">
      {visibleAnnotations.length}
    </span>
  </div>

  <!-- Annotation list -->
  <div class="flex-1 overflow-y-auto min-h-0">
    {#if visibleAnnotations.length === 0}
      <div class="px-3 py-8 text-center">
        <div
          class="w-10 h-10 rounded-xl bg-[var(--bg-sunken)] border border-[var(--border)]
            flex items-center justify-center text-xl mx-auto mb-3 select-none"
        >
          ✏️
        </div>
        <p class="text-sm text-[var(--text-muted)]">No annotations yet</p>
        <p class="text-xs text-[var(--text-faint)] mt-1">
          Draw on the canvas to add
        </p>
      </div>
    {:else}
      <div class="px-2 pb-3 space-y-0.5">
        {#each visibleAnnotations as ann (ann.id)}
          <div
            class="group flex items-start gap-2.5 px-2.5 py-2 rounded-lg cursor-pointer transition-colors
              {selectedId === ann.id
                ? 'bg-[var(--accent-subtle)]'
                : 'hover:bg-[var(--bg-surface-hover)]'}"
            style:box-shadow={selectedId === ann.id
              ? "inset 0 0 0 1.5px var(--accent)"
              : "none"}
            role="button"
            tabindex="0"
            onclick={() => handleAnnotationClick(ann)}
            ondblclick={() => handleDoubleClick(ann)}
            onkeydown={(e) => {
              if (e.key === "Enter") handleAnnotationClick(ann);
            }}
          >
            <!-- Number badge -->
            <span
              class="shrink-0 w-6 h-6 rounded-full flex items-center justify-center
                text-[11px] font-bold text-white mt-0.5"
              style:background-color={SEVERITY_COLORS[ann.severity] ??
                "#f97316"}
            >
              {ann.number}
            </span>

            <!-- Content -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span
                  class="text-[10px] font-medium px-1.5 py-0.5 rounded-md tracking-wide"
                  style:color={SEVERITY_COLORS[ann.severity]}
                  style:background-color={`${SEVERITY_COLORS[ann.severity] ?? "#f97316"}18`}
                >
                  {SEVERITY_LABELS[ann.severity] ?? ann.severity}
                </span>
                <span class="text-[11px] text-[var(--text-muted)] font-mono">
                  {ann.type}
                </span>
              </div>

              {#if ann.label}
                <p class="text-xs text-[var(--text-secondary)] mt-1 truncate">
                  {ann.label}
                </p>
              {/if}

              {#if ann.comment}
                <p class="text-xs text-[var(--text-muted)] mt-0.5 truncate">
                  {ann.comment}
                </p>
              {/if}
            </div>

            <!-- Delete button -->
            <button
              type="button"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded-md
                opacity-0 group-hover:opacity-100 text-[var(--text-muted)]
                hover:text-[var(--danger)] hover:bg-[var(--danger-subtle)]
                text-xs mt-0.5 transition-all"
              onclick={(e) => handleDelete(e, ann.id)}
              title="Delete annotation"
            >
              ✕
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</aside>

<!-- Comment popup -->
{#if editingAnnotation}
  <CommentPopup
    annotation={editingAnnotation}
    {annotationState}
    onclose={() => {
      editingAnnotation = null;
    }}
  />
{/if}
