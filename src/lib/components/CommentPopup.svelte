<script lang="ts">
  import type {
    Annotation,
    createAnnotationStore,
  } from "$lib/state/annotations.svelte";

  interface Props {
    annotation: Annotation;
    annotationState: ReturnType<typeof createAnnotationStore>;
    onclose: () => void;
  }

  let { annotation, annotationState, onclose }: Props = $props();

  // Local copies for editing — intentionally capturing initial values
  // svelte-ignore state_referenced_locally
  let comment = $state(annotation.comment);
  // svelte-ignore state_referenced_locally
  let label = $state(annotation.label);

  function save() {
    annotationState.update(annotation.id, { comment, label });
    onclose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    } else if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      save();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/40 z-40" onmousedown={onclose}></div>

<!-- Popup -->
<div
  class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50
  w-80 bg-[#1a1a1a] border border-white/10 rounded-lg shadow-2xl shadow-black/50"
>
  <!-- Header -->
  <div
    class="flex items-center justify-between px-3 py-2 border-b border-white/[0.06]"
  >
    <span class="text-xs font-mono text-white/40">#{annotation.number}</span>
    <button
      type="button"
      class="text-white/30 hover:text-white/60 text-xs"
      onclick={onclose}
    >
      ✕
    </button>
  </div>

  <!-- Body -->
  <div class="p-3 space-y-3">
    <div>
      <label
        class="block text-[10px] font-mono uppercase tracking-wider text-white/30 mb-1"
        for="popup-label"
      >
        Label
      </label>
      <input
        id="popup-label"
        type="text"
        bind:value={label}
        placeholder="e.g. spacing, alignment..."
        class="w-full bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
          text-sm text-white/80 placeholder:text-white/20
          focus:outline-none focus:border-[#f97316]/40 focus:ring-1 focus:ring-[#f97316]/20
          transition-colors"
      >
    </div>

    <div>
      <label
        class="block text-[10px] font-mono uppercase tracking-wider text-white/30 mb-1"
        for="popup-comment"
      >
        Comment
      </label>
      <textarea
        id="popup-comment"
        bind:value={comment}
        placeholder="Describe the issue..."
        rows="3"
        class="w-full bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
          text-sm text-white/80 placeholder:text-white/20 resize-none
          focus:outline-none focus:border-[#f97316]/40 focus:ring-1 focus:ring-[#f97316]/20
          transition-colors"
      ></textarea>
    </div>
  </div>

  <!-- Footer -->
  <div
    class="flex items-center justify-end gap-2 px-3 py-2 border-t border-white/[0.06]"
  >
    <span class="text-[9px] text-white/20 font-mono mr-auto"
      >⌘+Enter to save</span
    >
    <button
      type="button"
      class="px-3 py-1 text-xs text-white/40 hover:text-white/60 rounded transition-colors"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      type="button"
      class="px-3 py-1 text-xs text-[#0a0a0a] bg-[#f97316] hover:bg-[#f97316]/90
        rounded font-medium transition-colors"
      onclick={save}
    >
      Save
    </button>
  </div>
</div>
