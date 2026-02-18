<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  interface Props {
    onshowpreview: () => void;
    promptMarkdown: string;
  }

  let { promptMarkdown, onshowpreview }: Props = $props();

  let copyFeedback: string | null = $state(null);

  async function copyPrompt() {
    try {
      await writeText(promptMarkdown);
      showFeedback("Copied!");
    } catch (err) {
      console.error("Copy failed:", err);
      showFeedback("Copy failed");
    }
  }

  async function exportJson() {
    try {
      await writeText(JSON.stringify({ prompt: promptMarkdown }, null, 2));
      showFeedback("JSON copied");
    } catch (err) {
      console.error("JSON export failed:", err);
    }
  }

  function showFeedback(msg: string) {
    copyFeedback = msg;
    setTimeout(() => {
      copyFeedback = null;
    }, 1500);
  }
</script>

<div
  class="flex items-center gap-1.5 px-3 py-2 bg-[var(--bg-surface)] border-t border-[var(--border)]
    select-none shrink-0"
  style="box-shadow: var(--shadow-xs);"
>
  <!-- Preview button -->
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
      class="text-xs font-medium text-[var(--accent)] animate-pulse select-none"
    >
      {copyFeedback}
    </span>
  {/if}

  <!-- JSON export -->
  <button
    type="button"
    class="px-3 py-1.5 text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-secondary)]
      bg-[var(--bg-sunken)] hover:bg-[var(--bg-surface-hover)] border border-[var(--border)]
      rounded-md transition-colors"
    onclick={exportJson}
    title="Copy session as JSON"
  >
    JSON
  </button>

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

  <!-- Send to AI (primary CTA) -->
  <button
    type="button"
    class="px-4 py-1.5 text-xs font-semibold text-[var(--text-on-accent)] bg-[var(--accent)]
      hover:bg-[var(--accent-hover)] rounded-md transition-colors"
    style="box-shadow: var(--shadow-sm);"
    onclick={copyPrompt}
    title="Copy and send to AI"
  >
    Send to AI
  </button>
</div>
