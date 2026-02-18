<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    onclose: () => void;
    onshowintegrations: () => void;
  }

  let { onclose, onshowintegrations }: Props = $props();

  // Settings sections
  type Section = "general" | "labels" | "integrations" | "advanced";
  let activeSection: Section = $state("general");

  // Settings values (loaded from config)
  let theme = $state("dark");
  let globalShortcut = $state("CmdOrCtrl+Shift+E");
  let captureSoundEnabled = $state(false);
  let sessionLimit = $state(200);
  let retentionDays = $state(30);
  let gitDiffInPrompt = $state(true);
  let promptPreview = $state(true);
  let compressionMaxRes = $state(1920);
  let compressionQuality = $state(85);
  let logLevel = $state("info");

  let saving = $state(false);
  let saveMessage: string | null = $state(null);
  let uninstalling = $state(false);

  const sections: { id: Section; label: string }[] = [
    { id: "general", label: "General" },
    { id: "labels", label: "Quick Labels" },
    { id: "integrations", label: "Integrations" },
    { id: "advanced", label: "Advanced" },
  ];

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }

  function showSaveMessage(msg: string) {
    saveMessage = msg;
    setTimeout(() => {
      saveMessage = null;
    }, 2000);
  }

  /** Load settings from config.json on mount */
  async function loadConfig() {
    try {
      const config = await invoke<Record<string, unknown>>("load_app_config");
      theme = (config.theme as string) ?? "dark";
      globalShortcut =
        (config.global_shortcut as string) ?? "CmdOrCtrl+Shift+E";
      captureSoundEnabled = (config.capture_sound as boolean) ?? false;
      sessionLimit = (config.session_limit as number) ?? 200;
      retentionDays = (config.session_retention_days as number) ?? 30;
      gitDiffInPrompt = (config.git_diff_in_prompt as boolean) ?? true;
      promptPreview = (config.prompt_preview as boolean) ?? true;
      compressionMaxRes = (config.compression_max_resolution as number) ?? 1920;
      compressionQuality = (config.compression_quality as number) ?? 85;
      logLevel = (config.log_level as string) ?? "info";
    } catch (err) {
      console.error("Failed to load config:", err);
    }
  }

  /** Save all settings to config.json */
  async function saveConfig() {
    saving = true;
    try {
      const config = await invoke<Record<string, unknown>>("load_app_config");
      config.theme = theme;
      config.global_shortcut = globalShortcut;
      config.capture_sound = captureSoundEnabled;
      config.session_limit = sessionLimit;
      config.session_retention_days = retentionDays;
      config.git_diff_in_prompt = gitDiffInPrompt;
      config.prompt_preview = promptPreview;
      config.compression_max_resolution = compressionMaxRes;
      config.compression_quality = compressionQuality;
      config.log_level = logLevel;
      await invoke("save_app_config", { config });
      showSaveMessage("Settings saved");
    } catch (err) {
      console.error("Failed to save config:", err);
      showSaveMessage("Failed to save");
    } finally {
      saving = false;
    }
  }

  async function handleUninstall() {
    uninstalling = true;
    try {
      const result = await invoke<string>("run_uninstall", { backup: true });
      showSaveMessage(result);
    } catch (err) {
      showSaveMessage(`Uninstall failed: ${err}`);
    } finally {
      uninstalling = false;
    }
  }

  loadConfig();
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/60 z-40" onmousedown={onclose}></div>

<!-- Panel -->
<div
  class="fixed top-8 left-1/2 -translate-x-1/2 z-50 w-[560px] max-h-[80vh]
    bg-[#1a1a1a] border border-white/10 rounded-lg shadow-2xl shadow-black/60
    flex flex-col overflow-hidden"
>
  <!-- Header -->
  <div
    class="flex items-center justify-between px-4 py-2 border-b border-white/[0.06] shrink-0"
  >
    <span class="text-xs font-mono text-white/40">Settings</span>
    <button
      type="button"
      class="text-white/30 hover:text-white/60 text-xs"
      onclick={onclose}
    >
      ✕
    </button>
  </div>

  <!-- Body: nav + content -->
  <div class="flex flex-1 min-h-0">
    <!-- Section nav -->
    <nav
      class="w-36 shrink-0 border-r border-white/[0.06] py-2 px-2 space-y-0.5"
    >
      {#each sections as section (section.id)}
        <button
          type="button"
          class="w-full text-left px-2 py-1.5 text-[11px] rounded transition-colors
            {activeSection === section.id
            ? 'text-[#f97316] bg-[#f97316]/10'
            : 'text-white/40 hover:text-white/60 hover:bg-white/[0.04]'}"
          onclick={() => {
            activeSection = section.id;
          }}
        >
          {section.label}
        </button>
      {/each}
    </nav>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      {#if activeSection === "general"}
        <!-- Theme -->
        <div class="space-y-1">
          <label
            class="text-[10px] text-white/30 font-mono uppercase tracking-wide"
            >Theme</label
          >
          <select
            bind:value={theme}
            class="w-full bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
              text-xs text-white/70 focus:outline-none focus:border-[#f97316]/30"
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System</option>
          </select>
        </div>

        <!-- Global Shortcut -->
        <div class="space-y-1">
          <label
            class="text-[10px] text-white/30 font-mono uppercase tracking-wide"
            >Global Shortcut</label
          >
          <input
            type="text"
            bind:value={globalShortcut}
            class="w-full bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
              text-xs text-white/70 font-mono focus:outline-none focus:border-[#f97316]/30"
            readonly
          >
          <p class="text-[9px] text-white/15">
            Press the shortcut to capture a screenshot
          </p>
        </div>

        <!-- Capture Sound -->
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={captureSoundEnabled}
            class="accent-[#f97316]"
          >
          <span class="text-xs text-white/50">Capture sound effect</span>
        </label>

        <!-- Prompt Preview -->
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={promptPreview}
            class="accent-[#f97316]"
          >
          <span class="text-xs text-white/50"
            >Show prompt preview before sending</span
          >
        </label>

        <!-- Git Diff -->
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={gitDiffInPrompt}
            class="accent-[#f97316]"
          >
          <span class="text-xs text-white/50">Include git diff in prompt</span>
        </label>
      {:else if activeSection === "labels"}
        <div class="space-y-2">
          <p class="text-[10px] text-white/30 font-mono">
            Quick labels will be configurable in a future update.
          </p>
          <div class="space-y-1">
            {#each ["spacing", "alignment", "color", "font", "overflow", "responsive", "z-index", "missing element"] as label (label)}
              <div
                class="px-2 py-1.5 rounded bg-white/[0.02] text-xs text-white/40 font-mono"
              >
                {label}
              </div>
            {/each}
          </div>
        </div>
      {:else if activeSection === "integrations"}
        <div class="space-y-3">
          <p class="text-[10px] text-white/30">
            Manage MCP connections to AI coding tools.
          </p>
          <button
            type="button"
            class="px-4 py-2 text-xs font-medium text-[#0a0a0a] bg-[#f97316]
              hover:bg-[#f97316]/90 rounded transition-colors"
            onclick={onshowintegrations}
          >
            Open Integration Hub
          </button>
        </div>
      {:else if activeSection === "advanced"}
        <!-- Session Limit -->
        <div class="space-y-1">
          <label
            class="text-[10px] text-white/30 font-mono uppercase tracking-wide"
            >Session Limit</label
          >
          <input
            type="number"
            bind:value={sessionLimit}
            min="10"
            max="1000"
            class="w-24 bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
              text-xs text-white/70 font-mono focus:outline-none focus:border-[#f97316]/30"
          >
          <p class="text-[9px] text-white/15">
            Oldest sessions auto-deleted beyond this limit
          </p>
        </div>

        <!-- Retention Days -->
        <div class="space-y-1">
          <label
            class="text-[10px] text-white/30 font-mono uppercase tracking-wide"
            >Retention (days)</label
          >
          <input
            type="number"
            bind:value={retentionDays}
            min="1"
            max="365"
            class="w-24 bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
              text-xs text-white/70 font-mono focus:outline-none focus:border-[#f97316]/30"
          >
        </div>

        <!-- Compression -->
        <div class="space-y-1">
          <label
            class="text-[10px] text-white/30 font-mono uppercase tracking-wide"
            >Max Image Resolution</label
          >
          <select
            bind:value={compressionMaxRes}
            class="w-full bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
              text-xs text-white/70 focus:outline-none focus:border-[#f97316]/30"
          >
            <option value={1280}>1280px</option>
            <option value={1920}>1920px (default)</option>
            <option value={2560}>2560px</option>
            <option value={3840}>3840px (original)</option>
          </select>
        </div>

        <!-- Log Level -->
        <div class="space-y-1">
          <label
            class="text-[10px] text-white/30 font-mono uppercase tracking-wide"
            >Log Level</label
          >
          <select
            bind:value={logLevel}
            class="w-full bg-white/[0.04] border border-white/[0.08] rounded px-2 py-1.5
              text-xs text-white/70 focus:outline-none focus:border-[#f97316]/30"
          >
            <option value="error">Error</option>
            <option value="warn">Warn</option>
            <option value="info">Info (default)</option>
            <option value="debug">Debug</option>
            <option value="trace">Trace</option>
          </select>
        </div>

        <!-- Danger Zone -->
        <div class="mt-6 pt-4 border-t border-red-500/10 space-y-2">
          <p
            class="text-[10px] text-red-500/40 font-mono uppercase tracking-wide"
          >
            Danger Zone
          </p>
          <button
            type="button"
            class="px-3 py-1.5 text-[11px] text-red-400/60 border border-red-500/20
              rounded hover:bg-red-500/10 transition-colors disabled:opacity-30"
            disabled={uninstalling}
            onclick={handleUninstall}
          >
            {uninstalling ? "Uninstalling..." : "Uninstall CodeEye"}
          </button>
          <p class="text-[9px] text-white/15">
            Removes all data, disconnects tools, and cleans up
          </p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Footer -->
  <div
    class="flex items-center justify-between px-4 py-2 border-t border-white/[0.06] shrink-0"
  >
    {#if saveMessage}
      <span class="text-[10px] font-mono text-[#f97316]">{saveMessage}</span>
    {:else}
      <span></span>
    {/if}
    <button
      type="button"
      class="px-4 py-1.5 text-xs font-medium text-[#0a0a0a] bg-[#f97316]
        hover:bg-[#f97316]/90 rounded transition-colors disabled:opacity-50"
      disabled={saving}
      onclick={saveConfig}
    >
      {saving ? "Saving..." : "Save"}
    </button>
  </div>
</div>
