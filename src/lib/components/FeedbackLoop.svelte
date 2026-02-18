<script lang="ts">
  import type { Annotation } from "$lib/state/annotations.svelte";

  interface Props {
    afterSrc: string;
    annotations: Annotation[];
    beforeSrc: string;
    onclose: () => void;
  }

  let { beforeSrc, afterSrc, annotations, onclose }: Props = $props();

  // Local resolved tracking (not persisted to annotation store)
  let resolvedIds = $state(new Set<string>());

  // View mode: side-by-side or slider overlay
  let mode: "sideBySide" | "slider" = $state("slider");

  // Slider position (0–100)
  let sliderPos = $state(50);
  let isDragging = $state(false);

  let resolvedCount = $derived(resolvedIds.size);
  let totalCount = $derived(annotations.length);

  const SEVERITY_COLORS: Record<string, string> = {
    critical: "#ef4444",
    minor: "#eab308",
    suggestion: "#3b82f6",
  };

  function toggleResolved(id: string) {
    const next = new Set(resolvedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    resolvedIds = next;
  }

  function handleSliderMove(e: MouseEvent) {
    if (!isDragging) {
      return;
    }
    const target = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - target.left, target.width));
    sliderPos = (x / target.width) * 100;
  }

  function handleSliderDown(e: MouseEvent) {
    isDragging = true;
    const target = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - target.left, target.width));
    sliderPos = (x / target.width) * 100;
  }

  function handleSliderUp() {
    isDragging = false;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} onmouseup={handleSliderUp} />

<!-- Overlay backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-stretch bg-black/60 backdrop-blur-sm"
  onmousedown|self={onclose}
>
  <div
    class="flex flex-1 m-4 rounded-xl overflow-hidden bg-[var(--bg-app)] border border-[var(--border)]"
    style="box-shadow: var(--shadow-lg);"
  >
    <!-- Main comparison area -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Header bar -->
      <div
        class="flex items-center justify-between px-4 py-2.5 border-b border-[var(--border)] bg-[var(--bg-surface)] shrink-0"
      >
        <div class="flex items-center gap-3">
          <h2 class="text-sm font-semibold text-[var(--text-primary)]">
            Feedback Loop
          </h2>
          <span
            class="text-[11px] font-mono px-2 py-0.5 rounded-md bg-[var(--bg-sunken)] text-[var(--text-muted)]"
          >
            {resolvedCount}/{totalCount}
            resolved
          </span>
        </div>

        <div class="flex items-center gap-2">
          <!-- Mode toggle -->
          <div
            class="flex rounded-md border border-[var(--border)] overflow-hidden"
          >
            <button
              type="button"
              class="px-2.5 py-1 text-[11px] font-medium transition-colors
                {mode === 'slider'
                ? 'bg-[var(--accent)] text-[var(--text-on-accent)]'
                : 'bg-[var(--bg-surface)] text-[var(--text-muted)] hover:text-[var(--text-secondary)]'}"
              onclick={() => {
                mode = "slider";
              }}
            >
              Slider
            </button>
            <button
              type="button"
              class="px-2.5 py-1 text-[11px] font-medium transition-colors border-l border-[var(--border)]
                {mode === 'sideBySide'
                ? 'bg-[var(--accent)] text-[var(--text-on-accent)]'
                : 'bg-[var(--bg-surface)] text-[var(--text-muted)] hover:text-[var(--text-secondary)]'}"
              onclick={() => {
                mode = "sideBySide";
              }}
            >
              Side by Side
            </button>
          </div>

          <button
            type="button"
            class="w-7 h-7 flex items-center justify-center rounded-md text-[var(--text-muted)]
              hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)] transition-colors"
            onclick={onclose}
            title="Close (Esc)"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Comparison viewport -->
      <div class="flex-1 min-h-0 relative bg-[var(--bg-sunken)]">
        {#if mode === "slider"}
          <!-- Slider overlay mode -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="absolute inset-0 select-none"
            style="cursor: ew-resize;"
            onmousedown={handleSliderDown}
            onmousemove={handleSliderMove}
          >
            <!-- After image (full, underneath) -->
            <img
              src={afterSrc}
              alt="After"
              class="absolute inset-0 w-full h-full object-contain"
              draggable="false"
            >

            <!-- Before image (clipped by slider) -->
            <div
              class="absolute inset-0 overflow-hidden"
              style="width: {sliderPos}%;"
            >
              <img
                src={beforeSrc}
                alt="Before"
                class="absolute inset-0 h-full object-contain"
                style="width: {100 / (sliderPos / 100)}%;"
                draggable="false"
              >
            </div>

            <!-- Slider line + handle -->
            <div
              class="absolute top-0 bottom-0 w-0.5 bg-white"
              style="left: {sliderPos}%; transform: translateX(-50%);
                box-shadow: 0 0 4px rgba(0,0,0,0.5);"
            >
              <div
                class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2
                  w-8 h-8 rounded-full bg-white border-2 border-[var(--accent)]
                  flex items-center justify-center"
                style="box-shadow: 0 2px 8px rgba(0,0,0,0.3);"
              >
                <span
                  class="text-[10px] text-[var(--accent)] font-bold select-none"
                  >⇔</span
                >
              </div>
            </div>

            <!-- Labels -->
            <span
              class="absolute top-3 left-3 text-[10px] font-semibold uppercase tracking-wider
                px-2 py-0.5 rounded bg-black/50 text-white backdrop-blur-sm"
            >
              Before
            </span>
            <span
              class="absolute top-3 right-3 text-[10px] font-semibold uppercase tracking-wider
                px-2 py-0.5 rounded bg-black/50 text-white backdrop-blur-sm"
            >
              After
            </span>
          </div>
        {:else}
          <!-- Side-by-side mode -->
          <div class="flex h-full gap-px bg-[var(--border)]">
            <div class="flex-1 relative bg-[var(--bg-sunken)]">
              <img
                src={beforeSrc}
                alt="Before"
                class="w-full h-full object-contain"
              >
              <span
                class="absolute top-3 left-3 text-[10px] font-semibold uppercase tracking-wider
                  px-2 py-0.5 rounded bg-black/50 text-white backdrop-blur-sm"
              >
                Before
              </span>
            </div>
            <div class="flex-1 relative bg-[var(--bg-sunken)]">
              <img
                src={afterSrc}
                alt="After"
                class="w-full h-full object-contain"
              >
              <span
                class="absolute top-3 right-3 text-[10px] font-semibold uppercase tracking-wider
                  px-2 py-0.5 rounded bg-black/50 text-white backdrop-blur-sm"
              >
                After
              </span>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Annotation checklist sidebar -->
    <aside
      class="w-64 border-l border-[var(--border)] bg-[var(--bg-surface)] flex flex-col shrink-0"
    >
      <div class="px-3 pt-3 pb-2 border-b border-[var(--border)]">
        <span
          class="text-[11px] font-medium uppercase tracking-wider text-[var(--text-muted)]"
        >
          Issues
        </span>
      </div>

      <div class="flex-1 overflow-y-auto min-h-0">
        {#if annotations.length === 0}
          <div class="px-3 py-8 text-center">
            <p class="text-sm text-[var(--text-muted)]">No annotations</p>
          </div>
        {:else}
          <div class="px-2 py-2 space-y-0.5">
            {#each annotations as ann (ann.id)}
              {@const isResolved = resolvedIds.has(ann.id)}
              <button
                type="button"
                class="w-full flex items-start gap-2.5 px-2.5 py-2 rounded-lg text-left transition-colors
                  {isResolved
                  ? 'bg-green-500/8 hover:bg-green-500/12'
                  : 'hover:bg-[var(--bg-surface-hover)]'}"
                onclick={() => toggleResolved(ann.id)}
              >
                <!-- Checkbox -->
                <span
                  class="shrink-0 w-5 h-5 mt-0.5 rounded border-2 flex items-center justify-center transition-colors
                    {isResolved
                    ? 'bg-green-500 border-green-500 text-white'
                    : 'border-[var(--border)] bg-[var(--bg-sunken)]'}"
                >
                  {#if isResolved}
                    <span class="text-[10px] font-bold">✓</span>
                  {/if}
                </span>

                <!-- Number badge -->
                <span
                  class="shrink-0 w-5 h-5 rounded-full flex items-center justify-center
                    text-[10px] font-bold text-white mt-0.5"
                  style:background-color={SEVERITY_COLORS[ann.severity] ??
                    "#f97316"}
                  style:opacity={isResolved ? "0.4" : "1"}
                >
                  {ann.number}
                </span>

                <!-- Label -->
                <div class="flex-1 min-w-0">
                  <p
                    class="text-xs truncate transition-colors
                      {isResolved
                      ? 'text-[var(--text-faint)] line-through'
                      : 'text-[var(--text-secondary)]'}"
                  >
                    {ann.label || ann.type}
                  </p>
                  {#if ann.comment}
                    <p
                      class="text-[11px] text-[var(--text-muted)] truncate mt-0.5"
                    >
                      {ann.comment}
                    </p>
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Progress bar -->
      {#if totalCount > 0}
        <div class="px-3 py-2.5 border-t border-[var(--border)]">
          <div class="h-1.5 rounded-full bg-[var(--bg-sunken)] overflow-hidden">
            <div
              class="h-full rounded-full bg-green-500 transition-all duration-300"
              style="width: {(resolvedCount / totalCount) * 100}%;"
            ></div>
          </div>
          <p class="text-[10px] text-[var(--text-faint)] mt-1.5 text-center">
            {#if resolvedCount === totalCount}
              All issues resolved
            {:else}
              {totalCount - resolvedCount}
              remaining
            {/if}
          </p>
        </div>
      {/if}
    </aside>
  </div>
</div>
