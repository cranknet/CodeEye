<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import WelcomeCard from "./WelcomeCard.svelte";

  interface Props {
    oncomplete: () => void;
  }

  let { oncomplete }: Props = $props();

  let permissionOk = $state(true);
  let checkingPermission = $state(true);

  async function checkPermissions() {
    try {
      permissionOk = await invoke<boolean>("check_capture_permission");
    } catch {
      permissionOk = true; // Non-macOS, assume OK
    } finally {
      checkingPermission = false;
    }
  }

  function handleDismiss() {
    oncomplete();
  }

  // Check permissions on mount
  checkPermissions();
</script>

{#if checkingPermission}
  <!-- Loading state -->
  <div class="fixed inset-0 z-50 bg-[#0a0a0a] flex items-center justify-center">
    <p class="text-xs text-white/30 font-mono animate-pulse">
      Checking permissions...
    </p>
  </div>
{:else if !permissionOk}
  <!-- Permission needed -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 bg-black/70 z-40"></div>
  <div
    class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50
      w-[380px] bg-[#1a1a1a] border border-white/10 rounded-xl
      shadow-2xl shadow-black/60 p-6 text-center space-y-4"
  >
    <h2 class="text-sm font-medium text-white/70">Permission Required</h2>
    <p class="text-xs text-white/40 leading-relaxed">
      CodeEye needs screen recording permission to capture screenshots. Please
      enable it in System Settings &rarr; Privacy & Security &rarr; Screen
      Recording.
    </p>
    <div class="flex justify-center gap-2">
      <button
        type="button"
        class="px-4 py-1.5 text-xs text-white/40 hover:text-white/60 rounded transition-colors"
        onclick={handleDismiss}
      >
        Skip for now
      </button>
      <button
        type="button"
        class="px-4 py-1.5 text-xs font-medium text-[#0a0a0a] bg-[#f97316]
          hover:bg-[#f97316]/90 rounded transition-colors"
        onclick={() => {
          checkPermissions();
        }}
      >
        Check Again
      </button>
    </div>
  </div>
{:else}
  <!-- Welcome Card flow -->
  <WelcomeCard ondismiss={handleDismiss} />
{/if}
