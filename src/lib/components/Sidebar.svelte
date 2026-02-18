<script lang="ts">
  import type {
    Annotation,
    createAnnotationStore,
  } from "$lib/state/annotations.svelte";
  import CommentPopup from "./CommentPopup.svelte";

  interface Props {
    annotationState: ReturnType<typeof createAnnotationStore>;
    generalNotes: string;
    onnoteschange: (notes: string) => void;
    onpagechange: (name: string) => void;
    onselect: (id: string | null) => void;
    pageName: string;
    selectedId: string | null;
  }

  let {
    annotationState,
    selectedId,
    onselect,
    pageName,
    generalNotes,
    onpagechange,
    onnoteschange,
  }: Props = $props();

  let editingAnnotation: Annotation | null = $state(null);

  const SEVERITY_COLORS: Record<string, string> = {
    critical: "#ef4444",
    minor: "#eab308",
    suggestion: "#3b82f6",
  };

  const SEVERITY_LABELS: Record<string, string> = {
    critical: "CRT",
    minor: "MIN",
    suggestion: "SUG",
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
  class="w-64 bg-[#111111] border-l border-white/[0.06] flex flex-col h-full select-none overflow-hidden"
>
  <!-- Page info -->
  <div class="p-3 space-y-2 border-b border-white/[0.06] shrink-0">
    <div>
      <label
        class="block text-[10px] font-mono uppercase tracking-wider text-white/30 mb-1"
        for="page-name"
        >Page name</label
      >
      <input
        id="page-name"
        type="text"
        value={pageName}
        oninput={(e) => onpagechange((e.target as HTMLInputElement).value)}
        placeholder="e.g. /dashboard, LoginForm"
        class="w-full bg-white/[0.03] border border-white/[0.06] rounded px-2 py-1
          text-xs text-white/70 placeholder:text-white/15
          focus:outline-none focus:border-[#f97316]/30 transition-colors"
      >
    </div>

    <div>
      <label
        class="block text-[10px] font-mono uppercase tracking-wider text-white/30 mb-1"
        for="general-notes"
        >Notes</label
      >
      <textarea
        id="general-notes"
        value={generalNotes}
        oninput={(e) => onnoteschange((e.target as HTMLTextAreaElement).value)}
        placeholder="General notes about this page..."
        rows="2"
        class="w-full bg-white/[0.03] border border-white/[0.06] rounded px-2 py-1
          text-xs text-white/70 placeholder:text-white/15 resize-none
          focus:outline-none focus:border-[#f97316]/30 transition-colors"
      ></textarea>
    </div>
  </div>

  <!-- Annotation list header -->
  <div class="flex items-center justify-between px-3 py-2 shrink-0">
    <span class="text-[10px] font-mono uppercase tracking-wider text-white/30">
      Annotations
    </span>
    <span class="text-[10px] font-mono text-white/20">
      {annotationState.annotations.length}
    </span>
  </div>

  <!-- Annotation list -->
  <div class="flex-1 overflow-y-auto min-h-0">
    {#if annotationState.annotations.length === 0}
      <div class="px-3 py-6 text-center">
        <p class="text-[11px] text-white/20">No annotations yet</p>
        <p class="text-[10px] text-white/10 mt-1">Draw on the canvas to add</p>
      </div>
    {:else}
      <div class="px-1.5 pb-2 space-y-0.5">
        {#each annotationState.annotations as ann (ann.id)}
          <div
            class="group flex items-start gap-2 px-2 py-1.5 rounded-md cursor-pointer transition-colors
              {selectedId === ann.id
                ? 'bg-white/[0.06]'
                : 'hover:bg-white/[0.03]'}"
            role="button"
            tabindex="0"
            onclick={() => handleAnnotationClick(ann)}
            ondblclick={() => handleDoubleClick(ann)}
            onkeydown={(e) => { if (e.key === 'Enter') handleAnnotationClick(ann); }}
          >
            <!-- Number badge -->
            <span
              class="shrink-0 w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold text-white mt-0.5"
              style:background-color={SEVERITY_COLORS[ann.severity] ?? '#f97316'}
            >
              {ann.number}
            </span>

            <!-- Content -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <span
                  class="text-[9px] font-mono uppercase tracking-wider px-1 py-0.5 rounded"
                  style:color={SEVERITY_COLORS[ann.severity]}
                  style:background-color={`${SEVERITY_COLORS[ann.severity] ?? '#f97316'}15`}
                >
                  {SEVERITY_LABELS[ann.severity] ?? ann.severity}
                </span>
                <span class="text-[10px] text-white/30 font-mono"
                  >{ann.type}</span
                >
              </div>

              {#if ann.label}
                <p class="text-[11px] text-white/60 mt-0.5 truncate">
                  {ann.label}
                </p>
              {/if}

              {#if ann.comment}
                <p class="text-[10px] text-white/30 mt-0.5 truncate">
                  {ann.comment}
                </p>
              {/if}
            </div>

            <!-- Delete button -->
            <button
              type="button"
              class="shrink-0 opacity-0 group-hover:opacity-40 hover:!opacity-80
                text-white/50 text-[10px] mt-1 transition-opacity"
              onclick={(e) => handleDelete(e, ann.id)}
              title="Delete"
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
    onclose={() => { editingAnnotation = null; }}
  />
{/if}
