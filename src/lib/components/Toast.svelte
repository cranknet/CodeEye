<script lang="ts">
  import type { createToastStore } from "$lib/state/toast.svelte";

  interface Props {
    toastState: ReturnType<typeof createToastStore>;
  }

  let { toastState }: Props = $props();

  function typeColor(type: "success" | "error" | "info"): string {
    if (type === "success") {
      return "#22c55e";
    }
    if (type === "error") {
      return "#ef4444";
    }
    return "#f97316";
  }
</script>

{#if toastState.toasts.length > 0}
  <div
    class="fixed bottom-4 right-4 z-[60] flex flex-col gap-1.5 pointer-events-none"
  >
    {#each toastState.toasts as toast (toast.id)}
      <div
        class="pointer-events-auto px-3 py-2 rounded-md bg-[#1a1a1a] border border-white/[0.08]
          shadow-lg shadow-black/40 text-xs font-mono text-white/60
          animate-[slideIn_0.2s_ease-out]"
      >
        <span
          class="inline-block w-1.5 h-1.5 rounded-full mr-2"
          style:background-color={typeColor(toast.type)}
        ></span>
        {toast.text}
      </div>
    {/each}
  </div>
{/if}
