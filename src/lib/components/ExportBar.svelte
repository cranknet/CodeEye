<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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
      showFeedback("Prompt copied");
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
  class="flex items-center gap-1.5 px-3 py-1.5 bg-[#111111] border-t border-white/[0.06] select-none"
>
  <!-- Preview/Edit -->
  <button
    type="button"
    class="px-3 py-1 text-[11px] font-mono text-white/50 hover:text-white/80
      bg-white/[0.04] hover:bg-white/[0.08] rounded transition-colors"
    onclick={onshowpreview}
  >
    Preview
  </button>

  <div class="flex-1"></div>

  <!-- Feedback -->
  {#if copyFeedback}
    <span class="text-[10px] font-mono text-[#f97316] animate-pulse">
      {copyFeedback}
    </span>
  {/if}

  <!-- Copy Prompt -->
  <button
    type="button"
    class="px-3 py-1 text-[11px] font-mono text-white/50 hover:text-white/80
      bg-white/[0.04] hover:bg-white/[0.08] rounded transition-colors"
    onclick={copyPrompt}
    title="Copy prompt to clipboard"
  >
    Copy Prompt
  </button>

  <!-- Export JSON -->
  <button
    type="button"
    class="px-3 py-1 text-[11px] font-mono text-white/50 hover:text-white/80
      bg-white/[0.04] hover:bg-white/[0.08] rounded transition-colors"
    onclick={exportJson}
    title="Copy session as JSON"
  >
    JSON
  </button>

  <!-- Send -->
  <button
    type="button"
    class="px-3 py-1 text-[11px] font-medium text-[#0a0a0a] bg-[#f97316]
      hover:bg-[#f97316]/90 rounded transition-colors"
    onclick={copyPrompt}
    title="Copy and send to AI"
  >
    Send
  </button>
</div>
