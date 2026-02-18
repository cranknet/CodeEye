<script lang="ts">
  import type { createSessionStore } from "$lib/state/sessions.svelte";

  interface Props {
    onclose: () => void;
    onrestore: (sessionId: string) => void;
    sessionState: ReturnType<typeof createSessionStore>;
  }

  let { sessionState, onrestore, onclose }: Props = $props();

  let filter = $state("");

  let filteredSessions = $derived(
    filter
      ? sessionState.sessions.filter((s) =>
          s.pageName.toLowerCase().includes(filter.toLowerCase())
        )
      : sessionState.sessions
  );

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60_000);

    if (diffMins < 1) {
      return "just now";
    }
    if (diffMins < 60) {
      return `${diffMins}m ago`;
    }

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) {
      return `${diffHours}h ago`;
    }

    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) {
      return `${diffDays}d ago`;
    }

    return date.toLocaleDateString();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }

  async function handleDelete(e: MouseEvent, id: string) {
    e.stopPropagation();
    await sessionState.deleteSession(id);
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/60 z-40" onmousedown={onclose}></div>

<!-- Panel -->
<div
  class="fixed top-8 left-1/2 -translate-x-1/2 z-50 w-[500px] max-h-[80vh]
    bg-[#1a1a1a] border border-white/10 rounded-lg shadow-2xl shadow-black/60
    flex flex-col overflow-hidden"
>
  <!-- Header -->
  <div
    class="flex items-center justify-between px-4 py-2 border-b border-white/[0.06] shrink-0"
  >
    <span class="text-xs font-mono text-white/40">Sessions</span>
    <button
      type="button"
      class="text-white/30 hover:text-white/60 text-xs"
      onclick={onclose}
    >
      ✕
    </button>
  </div>

  <!-- Search -->
  <div class="px-4 py-2 border-b border-white/[0.06] shrink-0">
    <input
      type="text"
      bind:value={filter}
      placeholder="Filter sessions..."
      class="w-full bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
        text-xs text-white/70 placeholder:text-white/20
        focus:outline-none focus:border-[#f97316]/30 transition-colors"
    >
  </div>

  <!-- Session list -->
  <div class="flex-1 overflow-y-auto min-h-0">
    {#if filteredSessions.length === 0}
      <div class="px-4 py-8 text-center">
        <p class="text-[11px] text-white/20">No sessions found</p>
      </div>
    {:else}
      <div class="p-2 space-y-0.5">
        {#each filteredSessions as session (session.id)}
          <div
            class="group flex items-center gap-3 px-3 py-2 rounded-md cursor-pointer
              hover:bg-white/[0.04] transition-colors"
            role="button"
            tabindex="0"
            onclick={() => onrestore(session.id)}
            onkeydown={(e) => {
              if (e.key === "Enter") onrestore(session.id);
            }}
          >
            <!-- Status dot -->
            <span
              class="shrink-0 w-2 h-2 rounded-full"
              style:background-color={session.status === "open"
                ? "#f97316"
                : "#22c55e"}
            ></span>

            <!-- Info -->
            <div class="flex-1 min-w-0">
              <p class="text-xs text-white/60 truncate">
                {session.pageName || "Untitled"}
              </p>
              <p class="text-[10px] text-white/25 font-mono">
                {formatDate(session.createdAt)}
              </p>
            </div>

            <!-- Delete -->
            <button
              type="button"
              class="shrink-0 opacity-0 group-hover:opacity-40 hover:!opacity-80
                text-white/50 text-[10px] transition-opacity"
              onclick={(e) => handleDelete(e, session.id)}
              title="Delete"
            >
              ✕
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
