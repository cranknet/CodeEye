<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  interface AdapterStatus {
    config_path: string | null;
    connected: boolean;
    installed: boolean;
    tool_name: string;
  }

  let integrations: AdapterStatus[] = $state([]);
  let loading = $state(true);
  let actionInProgress: string | null = $state(null);

  async function scan() {
    loading = true;
    try {
      integrations = await invoke<AdapterStatus[]>("scan_integrations");
    } catch (err) {
      console.error("Failed to scan integrations:", err);
    } finally {
      loading = false;
    }
  }

  async function toggleConnection(toolName: string, connected: boolean) {
    actionInProgress = toolName;
    try {
      if (connected) {
        await invoke("disconnect_integration", { toolName });
      } else {
        await invoke("connect_integration", { toolName });
      }
      await scan();
    } catch (err) {
      console.error("Integration toggle failed:", err);
    } finally {
      actionInProgress = null;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }

  function statusColor(integration: AdapterStatus): string {
    if (integration.connected) {
      return "#22c55e";
    }
    if (integration.installed) {
      return "#eab308";
    }
    return "#666";
  }

  // Scan on mount
  scan();
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/60 z-40" onmousedown={onclose}></div>

<!-- Panel -->
<div
  class="fixed top-8 left-1/2 -translate-x-1/2 z-50 w-[480px] max-h-[80vh]
    bg-[#1a1a1a] border border-white/10 rounded-lg shadow-2xl shadow-black/60
    flex flex-col overflow-hidden"
>
  <!-- Header -->
  <div
    class="flex items-center justify-between px-4 py-3 border-b border-white/[0.06] shrink-0"
  >
    <div>
      <h2 class="text-sm font-medium text-white/80">Integrations</h2>
      <p class="text-[10px] text-white/30 font-mono mt-0.5">
        Connect AI tools to receive CodeEye feedback via MCP
      </p>
    </div>
    <button
      type="button"
      class="text-white/30 hover:text-white/60 text-xs"
      onclick={onclose}
    >
      ✕
    </button>
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-y-auto min-h-0 p-4 space-y-2">
    {#if loading}
      <div class="py-8 text-center">
        <p class="text-xs text-white/30 font-mono animate-pulse">
          Scanning for AI tools...
        </p>
      </div>
    {:else if integrations.length === 0}
      <div class="py-8 text-center">
        <p class="text-xs text-white/30">No AI tools detected</p>
      </div>
    {:else}
      {#each integrations as integration (integration.tool_name)}
        <div
          class="flex items-center gap-3 px-3 py-3 rounded-md bg-white/[0.02] border border-white/[0.06]"
        >
          <!-- Status dot -->
          <span
            class="shrink-0 w-2.5 h-2.5 rounded-full"
            style:background-color={statusColor(integration)}
          ></span>

          <!-- Info -->
          <div class="flex-1 min-w-0">
            <p class="text-xs text-white/70 font-medium">
              {integration.tool_name}
            </p>
            <p class="text-[10px] text-white/25 font-mono truncate">
              {#if integration.connected}
                Connected
              {:else if integration.installed}
                Installed — not connected
              {:else}
                Not installed
              {/if}
            </p>
          </div>

          <!-- Action button -->
          {#if integration.installed}
            <button
              type="button"
              class="shrink-0 px-3 py-1 text-[10px] font-mono rounded transition-colors
                {integration.connected
                ? 'text-white/40 bg-white/[0.04] hover:bg-white/[0.08]'
                : 'text-[#0a0a0a] bg-[#f97316] hover:bg-[#f97316]/90'}"
              disabled={actionInProgress === integration.tool_name}
              onclick={() =>
                toggleConnection(
                  integration.tool_name,
                  integration.connected,
                )}
            >
              {#if actionInProgress === integration.tool_name}
                ...
              {:else if integration.connected}
                Disconnect
              {:else}
                Connect
              {/if}
            </button>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Footer -->
  <div
    class="px-4 py-2 border-t border-white/[0.06] shrink-0 flex items-center justify-between"
  >
    <button
      type="button"
      class="text-[10px] text-white/25 hover:text-white/50 font-mono transition-colors"
      onclick={scan}
    >
      Rescan
    </button>
    <p class="text-[9px] text-white/15 font-mono">MCP stdio protocol</p>
  </div>
</div>
