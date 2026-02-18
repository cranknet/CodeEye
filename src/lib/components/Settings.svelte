<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    onclose: () => void;
    onshowintegrations: () => void;
    onthemechange?: (theme: string) => void;
  }

  let { onclose, onshowintegrations, onthemechange }: Props = $props();

  type Section = "general" | "labels" | "integrations" | "advanced";
  let activeSection: Section = $state("general");

  // ── General settings ──────────────────────────────────
  let theme = $state("light");
  let globalShortcut = $state("CmdOrCtrl+Shift+E");
  let launchAtLogin = $state(false);
  let captureSoundEnabled = $state(false);
  let promptPreview = $state(true);
  let gitDiffInPrompt = $state(true);
  let feedbackLoopEnabled = $state(false);

  // ── Advanced settings ─────────────────────────────────
  let sessionLimit = $state(200);
  let retentionDays = $state(30);
  let compressionMaxRes = $state(1920);
  let compressionFormat = $state("png");
  let compressionQuality = $state(85);
  let logLevel = $state("info");

  // ── UI state ──────────────────────────────────────────
  let saving = $state(false);
  let saveMessage: string | null = $state(null);
  let uninstalling = $state(false);
  let exportingDiagnostics = $state(false);
  let showUninstallConfirm = $state(false);

  const sections: { id: Section; label: string; icon: string }[] = [
    { id: "general", label: "General", icon: "◎" },
    { id: "labels", label: "Labels", icon: "⊞" },
    { id: "integrations", label: "Integrations", icon: "⊕" },
    { id: "advanced", label: "Advanced", icon: "⚙" },
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
    }, 2500);
  }

  async function loadConfig() {
    try {
      const config = await invoke<Record<string, unknown>>("load_app_config");
      theme = (config.theme as string) ?? "light";
      globalShortcut =
        (config.global_shortcut as string) ?? "CmdOrCtrl+Shift+E";
      launchAtLogin = (config.launch_at_login as boolean) ?? false;
      captureSoundEnabled = (config.capture_sound as boolean) ?? false;
      promptPreview = (config.prompt_preview as boolean) ?? true;
      gitDiffInPrompt = (config.git_diff_in_prompt as boolean) ?? true;
      feedbackLoopEnabled = (config.feedback_loop_enabled as boolean) ?? false;
      sessionLimit = (config.session_limit as number) ?? 200;
      retentionDays = (config.session_retention_days as number) ?? 30;
      compressionMaxRes = (config.compression_max_resolution as number) ?? 1920;
      compressionFormat = (config.compression_format as string) ?? "png";
      compressionQuality = (config.compression_quality as number) ?? 85;
      logLevel = (config.log_level as string) ?? "info";
    } catch (err) {
      console.error("Failed to load config:", err);
    }
  }

  async function saveConfig() {
    saving = true;
    try {
      const config = await invoke<Record<string, unknown>>("load_app_config");
      config.theme = theme;
      config.global_shortcut = globalShortcut;
      config.launch_at_login = launchAtLogin;
      config.capture_sound = captureSoundEnabled;
      config.prompt_preview = promptPreview;
      config.git_diff_in_prompt = gitDiffInPrompt;
      config.feedback_loop_enabled = feedbackLoopEnabled;
      config.session_limit = sessionLimit;
      config.session_retention_days = retentionDays;
      config.compression_max_resolution = compressionMaxRes;
      config.compression_format = compressionFormat;
      config.compression_quality = compressionQuality;
      config.log_level = logLevel;
      await invoke("save_app_config", { config });
      // Notify parent of theme change so it applies immediately
      onthemechange?.(theme);
      showSaveMessage("Settings saved");
    } catch (err) {
      console.error("Failed to save config:", err);
      showSaveMessage("Save failed");
    } finally {
      saving = false;
    }
  }

  async function exportDiagnostics() {
    exportingDiagnostics = true;
    try {
      await invoke("export_diagnostic_bundle");
      showSaveMessage("Diagnostic bundle exported");
    } catch (err) {
      showSaveMessage("Export failed — check logs");
    } finally {
      exportingDiagnostics = false;
    }
  }

  async function handleUninstall() {
    uninstalling = true;
    try {
      const result = await invoke<string>("run_uninstall", { backup: true });
      showSaveMessage(result);
      showUninstallConfirm = false;
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
<div
  class="fixed inset-0 z-40"
  style="background: var(--bg-overlay);"
  onmousedown={onclose}
></div>

<!-- Modal panel -->
<div
  class="fixed top-10 left-1/2 -translate-x-1/2 z-50 w-[640px] max-h-[82vh]
    bg-[var(--bg-surface)] border border-[var(--border)] rounded-xl
    flex flex-col overflow-hidden"
  style="box-shadow: var(--shadow-xl);"
>
  <!-- Header -->
  <div
    class="flex items-center justify-between px-5 py-3.5 border-b border-[var(--border)] shrink-0"
  >
    <span class="text-sm font-semibold text-[var(--text-primary)]"
      >Settings</span
    >
    <button
      type="button"
      class="w-7 h-7 flex items-center justify-center rounded-md text-xs
        text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)] transition-colors"
      onclick={onclose}
    >
      ✕
    </button>
  </div>

  <!-- Body: nav + content -->
  <div class="flex flex-1 min-h-0">
    <!-- Section nav -->
    <nav
      class="w-40 shrink-0 border-r border-[var(--border)] py-3 px-2 space-y-0.5 bg-[var(--bg-sunken)]"
    >
      {#each sections as section (section.id)}
        <button
          type="button"
          class="w-full flex items-center gap-2.5 px-2.5 py-2 text-sm rounded-md transition-colors
            {activeSection === section.id
              ? 'text-[var(--accent)] bg-[var(--accent-subtle)] font-medium'
              : 'text-[var(--text-muted)] hover:text-[var(--text-secondary)] hover:bg-[var(--bg-surface-hover)]'}"
          style:box-shadow={activeSection === section.id
            ? "inset 0 0 0 1.5px var(--accent)"
            : "none"}
          onclick={() => {
            activeSection = section.id;
          }}
        >
          <span class="text-base leading-none shrink-0">{section.icon}</span>
          {section.label}
        </button>
      {/each}
    </nav>

    <!-- Section content -->
    <div class="flex-1 overflow-y-auto p-5 space-y-5 min-h-0">
      <!-- ── General ─────────────────────────────── -->
      {#if activeSection === "general"}
        <!-- Theme -->
        <div class="space-y-2">
          <div>
            <label
              for="setting-theme"
              class="block text-xs font-semibold text-[var(--text-primary)] mb-0.5"
            >
              Theme
            </label>
            <p class="text-xs text-[var(--text-muted)]">
              Applies immediately on save
            </p>
          </div>
          <div class="flex gap-2">
            {#each ["light", "dark", "system"] as t}
              <button
                type="button"
                class="flex-1 py-2 text-xs font-medium rounded-md border transition-colors capitalize"
                style:background={theme === t
                  ? "var(--accent-subtle)"
                  : "var(--bg-sunken)"}
                style:color={theme === t
                  ? "var(--accent)"
                  : "var(--text-muted)"}
                style:border-color={theme === t ? "var(--accent)" : "var(--border)"}
                onclick={() => {
                  theme = t;
                }}
              >
                {t}
              </button>
            {/each}
          </div>
        </div>

        <!-- Global Shortcut -->
        <div class="space-y-1.5">
          <label
            for="setting-shortcut"
            class="block text-xs font-semibold text-[var(--text-primary)]"
          >
            Capture Shortcut
          </label>
          <input
            id="setting-shortcut"
            type="text"
            bind:value={globalShortcut}
            class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-3 py-2
              text-sm text-[var(--text-secondary)] font-mono
              focus:outline-none focus:border-[var(--border-focus)] transition-colors"
            readonly
          >
          <p class="text-xs text-[var(--text-muted)]">
            Press the shortcut anywhere to trigger a screenshot capture
          </p>
        </div>

        <!-- Toggles -->
        <div
          class="space-y-1 rounded-lg border border-[var(--border)] overflow-hidden"
        >
          {#each [
            { id: "launch-login", label: "Launch at login", desc: "Start CodeEye automatically when you log in", bind: "launchAtLogin" },
            { id: "capture-sound", label: "Capture sound effect", desc: "Play a sound when a screenshot is taken", bind: "captureSoundEnabled" },
            { id: "prompt-preview", label: "Show prompt preview", desc: "Review the generated prompt before sending to AI", bind: "promptPreview" },
            { id: "git-diff", label: "Include git diff in prompt", desc: "Attach recent file changes to the AI prompt", bind: "gitDiffInPrompt" },
            { id: "feedback-loop", label: "Enable feedback loop", desc: "Compare before/after screenshots to verify fixes", bind: "feedbackLoopEnabled" },
          ] as row, i}
            <label
              class="flex items-start gap-3 px-4 py-3 cursor-pointer transition-colors
                hover:bg-[var(--bg-surface-hover)]
                {i > 0 ? 'border-t border-[var(--border-subtle)]' : ''}"
              for={row.id}
            >
              <input
                id={row.id}
                type="checkbox"
                class="mt-0.5 accent-[#f97316] w-4 h-4 shrink-0 cursor-pointer"
                checked={row.bind === "launchAtLogin"
                  ? launchAtLogin
                  : row.bind === "captureSoundEnabled"
                    ? captureSoundEnabled
                    : row.bind === "promptPreview"
                      ? promptPreview
                      : row.bind === "gitDiffInPrompt"
                        ? gitDiffInPrompt
                        : feedbackLoopEnabled}
                onchange={(e) => {
                  const v = (e.target as HTMLInputElement).checked;
                  if (row.bind === "launchAtLogin") launchAtLogin = v;
                  else if (row.bind === "captureSoundEnabled")
                    captureSoundEnabled = v;
                  else if (row.bind === "promptPreview") promptPreview = v;
                  else if (row.bind === "gitDiffInPrompt") gitDiffInPrompt = v;
                  else feedbackLoopEnabled = v;
                }}
              >
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-[var(--text-primary)]">
                  {row.label}
                </p>
                <p class="text-xs text-[var(--text-muted)] mt-0.5">
                  {row.desc}
                </p>
              </div>
            </label>
          {/each}
        </div>
      <!-- ── Quick Labels ────────────────────────── -->
      {:else if activeSection === "labels"}
        <div>
          <p class="text-sm font-semibold text-[var(--text-primary)] mb-1">
            Quick Labels
          </p>
          <p class="text-xs text-[var(--text-muted)]">
            Click a label in the toolbar to pre-tag annotations. Custom label
            management coming in a future update.
          </p>
        </div>

        <div
          class="space-y-0.5 rounded-lg border border-[var(--border)] overflow-hidden"
        >
          {#each ["spacing", "alignment", "color", "font", "overflow", "responsive", "z-index", "missing element"] as label, i (label)}
            <div
              class="flex items-center justify-between px-4 py-2.5 text-sm
                text-[var(--text-secondary)]
                {i > 0 ? 'border-t border-[var(--border-subtle)]' : ''}"
            >
              <span class="font-mono text-xs">{label}</span>
              <span
                class="text-[10px] font-medium px-1.5 py-0.5 rounded bg-[var(--bg-sunken)]
                  text-[var(--text-muted)] border border-[var(--border)]"
              >
                default
              </span>
            </div>
          {/each}
        </div>
      <!-- ── Integrations ───────────────────────── -->
      {:else if activeSection === "integrations"}
        <div>
          <p class="text-sm font-semibold text-[var(--text-primary)] mb-1">
            AI Tool Integrations
          </p>
          <p class="text-xs text-[var(--text-muted)]">
            Connect CodeEye to your AI coding assistant via MCP (Model Context
            Protocol). Supported: Claude Code, Codex, Gemini CLI.
          </p>
        </div>

        <div
          class="p-4 rounded-lg border border-[var(--border)] bg-[var(--bg-sunken)] space-y-3"
        >
          <p class="text-xs text-[var(--text-muted)]">
            After connecting, your AI tools can read captured feedback sessions
            with
            <code
              class="px-1 py-0.5 rounded bg-[var(--bg-surface)] font-mono text-[var(--accent)]"
              >list_ui_feedback</code
            >
            and
            <code
              class="px-1 py-0.5 rounded bg-[var(--bg-surface)] font-mono text-[var(--accent)]"
              >get_ui_feedback</code
            >.
          </p>
          <button
            type="button"
            class="px-4 py-2 text-sm font-semibold text-[var(--text-on-accent)] bg-[var(--accent)]
              hover:bg-[var(--accent-hover)] rounded-md transition-colors"
            style="box-shadow: var(--shadow-sm);"
            onclick={onshowintegrations}
          >
            Open Integration Hub
          </button>
        </div>
      <!-- ── Advanced ───────────────────────────── -->
      {:else if activeSection === "advanced"}
        <!-- Session storage -->
        <div>
          <p
            class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-3"
          >
            Session Storage
          </p>
          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <label
                for="setting-session-limit"
                class="block text-xs font-medium text-[var(--text-secondary)]"
              >
                Session limit
              </label>
              <input
                id="setting-session-limit"
                type="number"
                bind:value={sessionLimit}
                min="10"
                max="1000"
                class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-3 py-2
                  text-sm text-[var(--text-primary)] font-mono
                  focus:outline-none focus:border-[var(--border-focus)] transition-colors"
              >
              <p class="text-xs text-[var(--text-muted)]">
                Oldest sessions auto-deleted beyond this count
              </p>
            </div>
            <div class="space-y-1.5">
              <label
                for="setting-retention"
                class="block text-xs font-medium text-[var(--text-secondary)]"
              >
                Retention (days)
              </label>
              <input
                id="setting-retention"
                type="number"
                bind:value={retentionDays}
                min="1"
                max="365"
                class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-3 py-2
                  text-sm text-[var(--text-primary)] font-mono
                  focus:outline-none focus:border-[var(--border-focus)] transition-colors"
              >
              <p class="text-xs text-[var(--text-muted)]">
                Sessions older than this are removed
              </p>
            </div>
          </div>
        </div>

        <!-- Compression -->
        <div>
          <p
            class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-3"
          >
            Image Compression
          </p>
          <div class="space-y-3">
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <label
                  for="setting-max-res"
                  class="block text-xs font-medium text-[var(--text-secondary)]"
                >
                  Max resolution
                </label>
                <select
                  id="setting-max-res"
                  bind:value={compressionMaxRes}
                  class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-3 py-2
                    text-sm text-[var(--text-primary)]
                    focus:outline-none focus:border-[var(--border-focus)] transition-colors"
                >
                  <option value={1280}>1280px</option>
                  <option value={1920}>1920px (default)</option>
                  <option value={2560}>2560px</option>
                  <option value={3840}>3840px (original)</option>
                </select>
              </div>
              <div class="space-y-1.5">
                <label
                  for="setting-format"
                  class="block text-xs font-medium text-[var(--text-secondary)]"
                >
                  Format
                </label>
                <select
                  id="setting-format"
                  bind:value={compressionFormat}
                  class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-3 py-2
                    text-sm text-[var(--text-primary)]
                    focus:outline-none focus:border-[var(--border-focus)] transition-colors"
                >
                  <option value="png">PNG (lossless)</option>
                  <option value="webp">WebP (smaller)</option>
                </select>
              </div>
            </div>

            <!-- Quality slider -->
            <div class="space-y-1.5">
              <div class="flex items-center justify-between">
                <label
                  for="setting-quality"
                  class="text-xs font-medium text-[var(--text-secondary)]"
                >
                  Quality
                </label>
                <span
                  class="text-xs font-mono font-semibold text-[var(--accent)]"
                >
                  {compressionQuality}%
                </span>
              </div>
              <input
                id="setting-quality"
                type="range"
                bind:value={compressionQuality}
                min="40"
                max="100"
                step="5"
                class="w-full accent-[#f97316] cursor-pointer"
              >
              <div
                class="flex justify-between text-[10px] text-[var(--text-faint)] font-mono"
              >
                <span>40% · smaller</span>
                <span>100% · best</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Logging & diagnostics -->
        <div>
          <p
            class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-3"
          >
            Logging & Diagnostics
          </p>
          <div class="space-y-3">
            <div class="space-y-1.5">
              <label
                for="setting-log-level"
                class="block text-xs font-medium text-[var(--text-secondary)]"
              >
                Log level
              </label>
              <select
                id="setting-log-level"
                bind:value={logLevel}
                class="w-full bg-[var(--bg-sunken)] border border-[var(--border)] rounded-md px-3 py-2
                  text-sm text-[var(--text-primary)]
                  focus:outline-none focus:border-[var(--border-focus)] transition-colors"
              >
                <option value="error">Error</option>
                <option value="warn">Warn</option>
                <option value="info">Info (default)</option>
                <option value="debug">Debug</option>
                <option value="trace">Trace</option>
              </select>
            </div>

            <div
              class="flex items-start gap-3 p-3 rounded-lg bg-[var(--bg-sunken)] border border-[var(--border)]"
            >
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-[var(--text-primary)]">
                  Export Diagnostic Bundle
                </p>
                <p class="text-xs text-[var(--text-muted)] mt-0.5">
                  Saves logs and config (no screenshots) to your desktop for
                  sharing with support
                </p>
              </div>
              <button
                type="button"
                class="shrink-0 px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)]
                  bg-[var(--bg-surface)] border border-[var(--border)] rounded-md
                  hover:bg-[var(--bg-surface-hover)] transition-colors disabled:opacity-50"
                disabled={exportingDiagnostics}
                onclick={exportDiagnostics}
              >
                {exportingDiagnostics ? "Exporting…" : "Export"}
              </button>
            </div>
          </div>
        </div>

        <!-- Danger Zone -->
        <div
          class="rounded-lg border overflow-hidden"
          style="border-color: var(--danger-border); background: var(--danger-subtle);"
        >
          <div
            class="px-4 py-3 border-b"
            style="border-color: var(--danger-border);"
          >
            <p
              class="text-xs font-semibold uppercase tracking-wider"
              style="color: var(--danger);"
            >
              Danger Zone
            </p>
          </div>

          <div class="px-4 py-3 space-y-3">
            {#if !showUninstallConfirm}
              <div class="flex items-start gap-3">
                <div class="flex-1">
                  <p class="text-sm font-medium text-[var(--text-primary)]">
                    Uninstall CodeEye
                  </p>
                  <p class="text-xs text-[var(--text-muted)] mt-0.5">
                    Removes all data, disconnects AI tools, cleans up config.
                    Creates a backup zip first.
                  </p>
                </div>
                <button
                  type="button"
                  class="shrink-0 px-3 py-1.5 text-xs font-medium rounded-md border transition-colors"
                  style="color: var(--danger); border-color: var(--danger-border); background: transparent;"
                  onclick={() => {
                    showUninstallConfirm = true;
                  }}
                >
                  Uninstall
                </button>
              </div>
            {:else}
              <p class="text-sm text-[var(--text-primary)]">
                Are you sure? This will permanently delete all sessions and
                config.
              </p>
              <div class="flex gap-2">
                <button
                  type="button"
                  class="px-3 py-1.5 text-xs font-semibold rounded-md transition-colors disabled:opacity-50"
                  style="background: var(--danger); color: var(--text-on-accent);"
                  disabled={uninstalling}
                  onclick={handleUninstall}
                >
                  {uninstalling ? "Uninstalling…" : "Yes, uninstall"}
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
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Footer -->
  <div
    class="flex items-center justify-between px-5 py-3 border-t border-[var(--border)] shrink-0
      bg-[var(--bg-sunken)]"
  >
    {#if saveMessage}
      <span class="text-xs font-medium text-[var(--accent)]"
        >{saveMessage}</span
      >
    {:else}
      <span></span>
    {/if}
    <div class="flex items-center gap-2">
      <button
        type="button"
        class="px-3 py-1.5 text-xs font-medium text-[var(--text-muted)]
          hover:text-[var(--text-secondary)] hover:bg-[var(--bg-surface-hover)]
          rounded-md transition-colors"
        onclick={onclose}
      >
        Cancel
      </button>
      <button
        type="button"
        class="px-4 py-1.5 text-xs font-semibold text-[var(--text-on-accent)] bg-[var(--accent)]
          hover:bg-[var(--accent-hover)] rounded-md transition-colors disabled:opacity-50"
        style="box-shadow: var(--shadow-sm);"
        disabled={saving}
        onclick={saveConfig}
      >
        {saving ? "Saving…" : "Save"}
      </button>
    </div>
  </div>
</div>
