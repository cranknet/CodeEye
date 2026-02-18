<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  interface Props {
    onclose: () => void;
    promptMarkdown: string;
  }

  let { promptMarkdown, onclose }: Props = $props();

  // Local editable copy
  // svelte-ignore state_referenced_locally
  let editablePrompt = $state(promptMarkdown);

  let copied = $state(false);

  async function copyAndClose() {
    try {
      await writeText(editablePrompt);
      copied = true;
      setTimeout(() => {
        onclose();
      }, 400);
    } catch (err) {
      console.error("Copy failed:", err);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    } else if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      copyAndClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/60 z-40" onmousedown={onclose}></div>

<!-- Panel -->
<div
  class="fixed top-8 left-1/2 -translate-x-1/2 z-50 w-[600px] max-h-[80vh]
    bg-[#1a1a1a] border border-white/10 rounded-lg shadow-2xl shadow-black/60
    flex flex-col overflow-hidden"
>
  <!-- Header -->
  <div
    class="flex items-center justify-between px-4 py-2 border-b border-white/[0.06] shrink-0"
  >
    <span class="text-xs font-mono text-white/40">Prompt Preview</span>
    <div class="flex items-center gap-2">
      <span class="text-[9px] text-white/20 font-mono">Cmd+Enter to send</span>
      <button
        type="button"
        class="text-white/30 hover:text-white/60 text-xs"
        onclick={onclose}
      >
        ✕
      </button>
    </div>
  </div>

  <!-- Editable prompt -->
  <div class="flex-1 overflow-y-auto min-h-0 p-4">
    <textarea
      bind:value={editablePrompt}
      class="w-full h-full min-h-[300px] bg-transparent text-sm text-white/70
        font-mono leading-relaxed resize-none outline-none
        placeholder:text-white/20"
      placeholder="No prompt generated yet..."
      spellcheck="false"
    ></textarea>
  </div>

  <!-- Footer -->
  <div
    class="flex items-center justify-between px-4 py-2 border-t border-white/[0.06] shrink-0"
  >
    <span class="text-[10px] font-mono text-white/20">
      {editablePrompt.length}
      chars
    </span>
    <div class="flex items-center gap-2">
      <button
        type="button"
        class="px-3 py-1 text-xs text-white/40 hover:text-white/60 rounded transition-colors"
        onclick={onclose}
      >
        Cancel
      </button>
      <button
        type="button"
        class="px-4 py-1 text-xs text-[#0a0a0a] bg-[#f97316] hover:bg-[#f97316]/90
          rounded font-medium transition-colors"
        onclick={copyAndClose}
      >
        {copied ? "Copied!" : "Copy & Send"}
      </button>
    </div>
  </div>
</div>
