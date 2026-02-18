<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    ondismiss: () => void;
  }

  let { ondismiss }: Props = $props();

  let step = $state(0);

  interface AdapterStatus {
    config_path: string | null;
    connected: boolean;
    installed: boolean;
    tool_name: string;
    verification: string | null;
  }

  let integrations: AdapterStatus[] = $state([]);

  let connecting: string | null = $state(null);

  async function scanTools() {
    try {
      integrations = await invoke<AdapterStatus[]>("scan_integrations");
    } catch (err) {
      console.error("Failed to scan:", err);
    }
  }

  async function quickConnect(toolName: string) {
    connecting = toolName;
    try {
      await invoke("connect_integration", { toolName });
      await scanTools();
    } catch (err) {
      console.error("Connect failed:", err);
    } finally {
      connecting = null;
    }
  }

  function toolDotColor(tool: AdapterStatus): string {
    if (tool.connected) {
      return "#22c55e";
    }
    if (tool.installed) {
      return "#eab308";
    }
    return "#444";
  }

  function next() {
    if (step < 2) {
      step++;
      if (step === 2) {
        scanTools();
      }
    } else {
      ondismiss();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      ondismiss();
    } else if (e.key === "Enter") {
      next();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/70 z-40" onmousedown={ondismiss}></div>

<!-- Card -->
<div
  class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50
    w-[420px] bg-[#1a1a1a] border border-white/10 rounded-xl
    shadow-2xl shadow-black/60 overflow-hidden"
>
  <!-- Progress dots -->
  <div class="flex justify-center gap-1.5 pt-4">
    {#each [0, 1, 2] as i (i)}
      <span
        class="w-1.5 h-1.5 rounded-full transition-colors"
        style:background-color={i <= step ? "#f97316" : "rgba(255,255,255,0.1)"}
      ></span>
    {/each}
  </div>

  <!-- Content -->
  <div class="px-6 py-6">
    {#if step === 0}
      <!-- Step 1: Welcome -->
      <div class="text-center">
        <h1 class="text-lg font-semibold text-[#f97316]">CodeEye</h1>
        <p class="text-xs text-white/50 mt-2 leading-relaxed">
          The fastest way to report UI bugs to AI. Capture screenshots, annotate
          issues, and send structured feedback directly to your AI assistant.
        </p>
      </div>
    {:else if step === 1}
      <!-- Step 2: How it works -->
      <div class="space-y-3">
        <h2 class="text-sm font-medium text-white/70 text-center">
          How it works
        </h2>
        <div class="space-y-2">
          {#each [
            {
              num: "1",
              title: "Capture",
              desc: "Press Ctrl+Shift+E to capture any region of your screen",
            },
            {
              num: "2",
              title: "Annotate",
              desc: "Circle issues, add labels, set severity levels",
            },
            {
              num: "3",
              title: "Send",
              desc: "Copy the structured prompt and paste to your AI tool",
            },
          ] as item (item.num)}
            <div class="flex items-start gap-3 px-3 py-2">
              <span
                class="shrink-0 w-5 h-5 rounded-full bg-[#f97316]/20 text-[#f97316]
                  text-[10px] font-bold flex items-center justify-center"
              >
                {item.num}
              </span>
              <div>
                <p class="text-xs text-white/60 font-medium">{item.title}</p>
                <p class="text-[10px] text-white/30">{item.desc}</p>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <!-- Step 3: Auto-detected tools -->
      <div class="space-y-3">
        <h2 class="text-sm font-medium text-white/70 text-center">
          AI Tools Detected
        </h2>
        <p class="text-[10px] text-white/30 text-center">
          Installed tools are auto-configured via MCP
        </p>
        <div class="space-y-1.5">
          {#each integrations as tool (tool.tool_name)}
            <div
              class="flex items-center gap-3 px-3 py-2 rounded-md bg-white/[0.02]"
            >
              <span
                class="w-2 h-2 rounded-full shrink-0"
                style:background-color={toolDotColor(tool)}
              ></span>
              <span class="text-xs text-white/50 flex-1">{tool.tool_name}</span>
              {#if tool.connected}
                <span class="text-[10px] text-green-500/60">Connected</span>
              {:else if tool.installed}
                <button
                  type="button"
                  class="px-2 py-0.5 text-[10px] text-[#0a0a0a] bg-[#f97316] rounded
                    hover:bg-[#f97316]/90 transition-colors"
                  disabled={connecting === tool.tool_name}
                  onclick={() => quickConnect(tool.tool_name)}
                >
                  {connecting === tool.tool_name ? "..." : "Connect"}
                </button>
              {:else}
                <span class="text-[10px] text-white/20">Not installed</span>
              {/if}
            </div>
          {/each}
          {#if integrations.length === 0}
            <p class="text-[10px] text-white/20 text-center py-2 animate-pulse">
              Scanning...
            </p>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <div class="px-6 pb-5 flex justify-between items-center">
    <button
      type="button"
      class="text-[10px] text-white/20 hover:text-white/40 transition-colors"
      onclick={ondismiss}
    >
      Skip
    </button>
    <button
      type="button"
      class="px-4 py-1.5 text-xs font-medium text-[#0a0a0a] bg-[#f97316]
        hover:bg-[#f97316]/90 rounded transition-colors"
      onclick={next}
    >
      {step === 2 ? "Get Started" : "Next"}
    </button>
  </div>
</div>
