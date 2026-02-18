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
    verification: string | null;
  }

  // ── State ──────────────────────────────────────────────
  let integrations: AdapterStatus[] = $state([]);
  let loading = $state(true);
  let selectedTool: string | null = $state(null);
  let actionInProgress: string | null = $state(null);

  // Detail panel state
  let entryContent: string | null = $state(null);
  let backupPath: string | null = $state(null);
  let verifyResult: string | null = $state(null);
  let verifying = $state(false);
  let showUninstallConfirm = $state(false);
  let statusMessage: string | null = $state(null);

  let selected = $derived(
    integrations.find((i) => i.tool_name === selectedTool) ?? null
  );

  // ── Actions ────────────────────────────────────────────
  async function scan() {
    loading = true;
    try {
      integrations = await invoke<AdapterStatus[]>("scan_integrations");
      if (integrations.length > 0 && !selectedTool) {
        selectedTool = integrations[0].tool_name;
      }
    } catch (err) {
      console.error("Failed to scan integrations:", err);
    } finally {
      loading = false;
    }
  }

  // Load detail data when selection changes
  $effect(() => {
    const tool = selectedTool;
    if (!tool) {
      return;
    }

    const integration = integrations.find((i) => i.tool_name === tool);
    verifyResult = null;
    showUninstallConfirm = false;
    statusMessage = null;

    if (!integration?.connected) {
      entryContent = null;
      backupPath = null;
      return;
    }

    invoke<string>("read_mcp_entry", { toolName: tool })
      .then((content) => {
        entryContent = content;
      })
      .catch(() => {
        entryContent = null;
      });

    invoke<string | null>("get_backup_path", { toolName: tool })
      .then((path) => {
        backupPath = path;
      })
      .catch(() => {
        backupPath = null;
      });
  });

  async function connect(toolName: string) {
    actionInProgress = toolName;
    try {
      await invoke("connect_integration", { toolName });
      showStatus("Connected");
      await scan();
    } catch (err) {
      showStatus(`Failed: ${err}`);
    } finally {
      actionInProgress = null;
    }
  }

  async function disconnect(toolName: string) {
    actionInProgress = toolName;
    try {
      await invoke("disconnect_integration", { toolName });
      showStatus("Disconnected");
      await scan();
    } catch (err) {
      showStatus(`Failed: ${err}`);
    } finally {
      actionInProgress = null;
    }
  }

  async function verify() {
    if (!selectedTool) {
      return;
    }
    verifying = true;
    verifyResult = null;
    try {
      verifyResult = await invoke<string>("verify_integration", {
        toolName: selectedTool,
      });
    } catch (err) {
      verifyResult = `Error: ${err}`;
    } finally {
      verifying = false;
    }
  }

  async function restoreBackup() {
    if (!selectedTool) {
      return;
    }
    actionInProgress = selectedTool;
    try {
      await invoke("restore_config_backup", { toolName: selectedTool });
      showStatus("Backup restored");
      await scan();
    } catch (err) {
      showStatus(`Restore failed: ${err}`);
    } finally {
      actionInProgress = null;
    }
  }

  async function uninstallMcp() {
    if (!selectedTool) {
      return;
    }
    actionInProgress = selectedTool;
    try {
      await invoke("uninstall_mcp", { toolName: selectedTool });
      showUninstallConfirm = false;
      showStatus("MCP removed");
      await scan();
    } catch (err) {
      showStatus(`Uninstall failed: ${err}`);
    } finally {
      actionInProgress = null;
    }
  }

  function showStatus(msg: string) {
    statusMessage = msg;
    setTimeout(() => {
      statusMessage = null;
    }, 3000);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }

  function statusDot(i: AdapterStatus): string {
    if (i.connected) {
      return "bg-emerald-500";
    }
    if (i.installed) {
      return "bg-amber-500";
    }
    return "bg-[var(--text-faint)]";
  }

  scan();
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-40"
  style="background: var(--bg-overlay);"
  onmousedown={onclose}
></div>

<!-- Panel -->
<div
  class="fixed top-8 left-1/2 -translate-x-1/2 z-50 w-[700px] max-h-[82vh]
    bg-[var(--bg-surface)] border border-[var(--border)] rounded-xl
    flex flex-col overflow-hidden"
  style="box-shadow: var(--shadow-xl);"
>
  <!-- Header -->
  <div
    class="flex items-center justify-between px-5 py-3.5 border-b border-[var(--border)] shrink-0"
  >
    <div>
      <h2 class="text-sm font-semibold text-[var(--text-primary)]">
        Integration Hub
      </h2>
      <p class="text-[10px] text-[var(--text-muted)] font-mono mt-0.5">
        Connect AI tools via MCP (Model Context Protocol)
      </p>
    </div>
    <button
      type="button"
      class="w-7 h-7 flex items-center justify-center rounded-md text-xs
        text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)] transition-colors"
      onclick={onclose}
    >
      ✕
    </button>
  </div>

  <!-- Body: two-panel -->
  <div class="flex flex-1 min-h-0">
    <!-- Left: tool list -->
    <nav
      class="w-44 shrink-0 border-r border-[var(--border)] py-3 px-2 space-y-0.5
        bg-[var(--bg-sunken)] overflow-y-auto"
    >
      {#if loading}
        <div class="py-6 text-center">
          <p class="text-xs text-[var(--text-faint)] animate-pulse">
            Scanning...
          </p>
        </div>
      {:else}
        {#each integrations as tool (tool.tool_name)}
          <button
            type="button"
            class="w-full flex items-center gap-2.5 px-2.5 py-2.5 text-sm rounded-md transition-colors
              {selectedTool === tool.tool_name
                ? 'text-[var(--text-primary)] bg-[var(--bg-surface)] font-medium border border-[var(--border)]'
                : 'text-[var(--text-muted)] hover:text-[var(--text-secondary)] hover:bg-[var(--bg-surface-hover)]'}"
            style:box-shadow={selectedTool === tool.tool_name
              ? "var(--shadow-xs)"
              : "none"}
            onclick={() => {
              selectedTool = tool.tool_name;
            }}
          >
            <span
              class="shrink-0 w-2 h-2 rounded-full {statusDot(tool)}"
            ></span>
            <span class="truncate text-xs">{tool.tool_name}</span>
          </button>
        {/each}
      {/if}
    </nav>

    <!-- Right: detail panel -->
    <div class="flex-1 overflow-y-auto p-5 space-y-5 min-h-0">
      {#if !selected}
        <div class="py-12 text-center">
          <p class="text-xs text-[var(--text-faint)]">
            Select a tool from the list
          </p>
        </div>
      <!-- ── Not installed ──────────────────────── -->
      {:else if !selected.installed}
        <div class="space-y-3">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full {statusDot(selected)}"></span>
            <h3 class="text-sm font-medium text-[var(--text-primary)]">
              {selected.tool_name}
            </h3>
          </div>

          <div
            class="p-4 rounded-lg bg-[var(--bg-sunken)] border border-[var(--border)]"
          >
            <p class="text-xs text-[var(--text-muted)] leading-relaxed">
              <span class="font-semibold text-[var(--text-secondary)]"
                >{selected.tool_name}</span
              >
              is not installed on this system. Install it first, then return
              here to connect CodeEye.
            </p>
          </div>
        </div>
      <!-- ── Installed, not connected ───────────── -->
      {:else if !selected.connected}
        <div class="space-y-4">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full {statusDot(selected)}"></span>
            <h3 class="text-sm font-medium text-[var(--text-primary)]">
              {selected.tool_name}
            </h3>
            <span
              class="text-[10px] font-mono text-amber-500 bg-amber-500/10 px-1.5 py-0.5 rounded"
            >
              installed
            </span>
          </div>

          {#if selected.config_path}
            <div class="space-y-1.5">
              <p
                class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
              >
                Config file
              </p>
              <p
                class="text-xs font-mono text-[var(--text-faint)] bg-[var(--bg-sunken)]
                  border border-[var(--border)] rounded-md px-3 py-2 break-all"
              >
                {selected.config_path}
              </p>
            </div>
          {/if}

          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold text-[var(--text-on-accent)] bg-[var(--accent)]
              hover:bg-[var(--accent-hover)] rounded-md transition-colors disabled:opacity-50"
            style="box-shadow: var(--shadow-sm);"
            disabled={actionInProgress === selected.tool_name}
            onclick={() => connect(selected.tool_name)}
          >
            {actionInProgress === selected.tool_name
              ? "Connecting..."
              : "Connect CodeEye"}
          </button>

          <p class="text-[10px] text-[var(--text-faint)] leading-relaxed">
            This will add the CodeEye MCP server entry to the tool's
            configuration file. A backup will be created automatically.
          </p>
        </div>
      <!-- ── Connected ──────────────────────────── -->
      {:else}
        <div class="space-y-5">
          <!-- Header -->
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full {statusDot(selected)}"></span>
            <h3 class="text-sm font-medium text-[var(--text-primary)]">
              {selected.tool_name}
            </h3>
            <span
              class="text-[10px] font-mono text-emerald-500 bg-emerald-500/10 px-1.5 py-0.5 rounded"
            >
              connected
            </span>
          </div>

          <!-- MCP Entry preview -->
          {#if entryContent}
            <div class="space-y-1.5">
              <p
                class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
              >
                MCP Entry
              </p>
              <pre
                class="text-[11px] font-mono text-[var(--text-secondary)] bg-[var(--bg-sunken)]
                  border border-[var(--border)] rounded-md px-3 py-2.5 overflow-x-auto
                  leading-relaxed max-h-36"
              >{entryContent}</pre>
            </div>
          {/if}

          <!-- Actions -->
          <div class="flex items-center gap-2">
            <button
              type="button"
              class="px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)]
                bg-[var(--bg-sunken)] hover:bg-[var(--bg-surface-hover)] border border-[var(--border)]
                rounded-md transition-colors disabled:opacity-50"
              disabled={verifying}
              onclick={verify}
            >
              {verifying ? "Verifying..." : "Verify"}
            </button>
            <button
              type="button"
              class="px-3 py-1.5 text-xs font-medium text-[var(--text-muted)]
                hover:text-red-400 bg-[var(--bg-sunken)] hover:bg-red-500/10
                border border-[var(--border)] hover:border-red-500/30
                rounded-md transition-colors disabled:opacity-50"
              disabled={actionInProgress === selected.tool_name}
              onclick={() => disconnect(selected.tool_name)}
            >
              {actionInProgress === selected.tool_name
                ? "..."
                : "Disconnect"}
            </button>
          </div>

          <!-- Verify result -->
          {#if verifyResult}
            <div
              class="text-xs font-mono px-3 py-2 rounded-md border
                {verifyResult.startsWith('Error')
                  ? 'text-red-400 bg-red-500/10 border-red-500/20'
                  : 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20'}"
            >
              {verifyResult}
            </div>
          {/if}

          <!-- Backup section -->
          {#if backupPath}
            <div class="space-y-2">
              <p
                class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
              >
                Config Backup
              </p>
              <div
                class="flex items-start gap-3 p-3 rounded-md bg-[var(--bg-sunken)] border border-[var(--border)]"
              >
                <div class="flex-1 min-w-0">
                  <p
                    class="text-xs font-mono text-[var(--text-faint)] break-all"
                  >
                    {backupPath}
                  </p>
                </div>
                <button
                  type="button"
                  class="shrink-0 px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)]
                    bg-[var(--bg-surface)] border border-[var(--border)] rounded-md
                    hover:bg-[var(--bg-surface-hover)] transition-colors disabled:opacity-50"
                  disabled={actionInProgress === selected.tool_name}
                  onclick={restoreBackup}
                >
                  Restore
                </button>
              </div>
            </div>
          {/if}

          <!-- Danger zone -->
          <div
            class="rounded-lg border overflow-hidden"
            style="border-color: var(--danger-border); background: var(--danger-subtle);"
          >
            <div
              class="px-3 py-2 border-b"
              style="border-color: var(--danger-border);"
            >
              <p
                class="text-[10px] font-semibold uppercase tracking-wider"
                style="color: var(--danger);"
              >
                Danger Zone
              </p>
            </div>
            <div class="px-3 py-3">
              {#if !showUninstallConfirm}
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-xs font-medium text-[var(--text-primary)]">
                      Remove MCP entry
                    </p>
                    <p class="text-[10px] text-[var(--text-muted)] mt-0.5">
                      Disconnect and delete the backup file
                    </p>
                  </div>
                  <button
                    type="button"
                    class="shrink-0 px-3 py-1.5 text-xs font-medium rounded-md border transition-colors"
                    style="color: var(--danger); border-color: var(--danger-border);"
                    onclick={() => {
                      showUninstallConfirm = true;
                    }}
                  >
                    Uninstall
                  </button>
                </div>
              {:else}
                <div class="space-y-2">
                  <p class="text-xs text-[var(--text-primary)]">
                    Remove CodeEye MCP entry from {selected.tool_name}?
                  </p>
                  <div class="flex gap-2">
                    <button
                      type="button"
                      class="px-3 py-1.5 text-xs font-semibold rounded-md transition-colors disabled:opacity-50"
                      style="background: var(--danger); color: var(--text-on-accent);"
                      disabled={actionInProgress === selected.tool_name}
                      onclick={uninstallMcp}
                    >
                      {actionInProgress === selected.tool_name
                        ? "Removing..."
                        : "Yes, remove"}
                    </button>
                    <button
                      type="button"
                      class="px-3 py-1.5 text-xs font-medium text-[var(--text-muted)]
                        bg-[var(--bg-surface)] border border-[var(--border)] rounded-md
                        hover:bg-[var(--bg-surface-hover)] transition-colors"
                      onclick={() => {
                        showUninstallConfirm = false;
                      }}
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Footer -->
  <div
    class="flex items-center justify-between px-5 py-2.5 border-t border-[var(--border)] shrink-0
      bg-[var(--bg-sunken)]"
  >
    <div class="flex items-center gap-3">
      <button
        type="button"
        class="text-[10px] text-[var(--text-muted)] hover:text-[var(--text-secondary)]
          font-mono transition-colors"
        onclick={scan}
      >
        Rescan
      </button>
      {#if statusMessage}
        <span class="text-[10px] font-medium text-[var(--accent)]">
          {statusMessage}
        </span>
      {/if}
    </div>
    <p class="text-[9px] text-[var(--text-faint)] font-mono">
      MCP stdio protocol
    </p>
  </div>
</div>
