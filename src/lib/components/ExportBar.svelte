<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  interface Props {
    onshowpreview: () => void;
    onsubmit: () => void;
    promptMarkdown: string;
  }

  let { promptMarkdown, onshowpreview, onsubmit }: Props = $props();

  let submitState: "idle" | "submitting" | "done" = $state("idle");
  let copyFeedback: string | null = $state(null);

  async function copyPrompt() {
    try {
      await writeText(promptMarkdown);
      copyFeedback = "Copied!";
    } catch (err) {
      console.error("Copy failed:", err);
      copyFeedback = "Copy failed";
    }
    setTimeout(() => {
      copyFeedback = null;
    }, 1500);
  }

  async function handleSubmit() {
    if (submitState !== "idle") {
      return;
    }
    submitState = "submitting";
    try {
      await Promise.resolve(onsubmit());
      submitState = "done";
      setTimeout(() => {
        submitState = "idle";
      }, 2500);
    } catch {
      submitState = "idle";
    }
  }
</script>

<div
  class="flex items-center gap-1.5 px-3 py-2 bg-[var(--bg-surface)] border-t border-[var(--border)]
    select-none shrink-0"
  style="box-shadow: var(--shadow-xs);"
>
  <!-- Preview -->
  <button
    type="button"
    class="px-3 py-1.5 text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-secondary)]
      bg-[var(--bg-sunken)] hover:bg-[var(--bg-surface-hover)] border border-[var(--border)]
      rounded-md transition-colors"
    onclick={onshowpreview}
  >
    Preview
  </button>

  <div class="flex-1"></div>

  <!-- Copy feedback -->
  {#if copyFeedback}
    <span
      class="text-xs font-medium text-[var(--accent)] select-none transition-opacity"
    >
      {copyFeedback}
    </span>
  {/if}

  <!-- Copy Prompt -->
  <button
    type="button"
    class="px-3 py-1.5 text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-secondary)]
      bg-[var(--bg-sunken)] hover:bg-[var(--bg-surface-hover)] border border-[var(--border)]
      rounded-md transition-colors"
    onclick={copyPrompt}
    title="Copy prompt to clipboard"
  >
    Copy Prompt
  </button>

  <!-- Submit Feedback (primary CTA) -->
  <button
    type="button"
    class="px-4 py-1.5 text-xs font-semibold rounded-md transition-all
      {submitState === 'done'
        ? 'bg-emerald-600 text-white'
        : 'text-[var(--text-on-accent)] bg-[var(--accent)] hover:bg-[var(--accent-hover)]'}
      disabled:opacity-60"
    style="box-shadow: var(--shadow-sm); min-width: 120px;"
    disabled={submitState === "submitting"}
    onclick={handleSubmit}
  >
    {#if submitState === "submitting"}
      <span class="inline-flex items-center gap-1.5">
        <span
          class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"
        ></span>
        Submitting
      </span>
    {:else if submitState === "done"}
      Submitted
    {:else}
      Submit Feedback
    {/if}
  </button>
</div>
