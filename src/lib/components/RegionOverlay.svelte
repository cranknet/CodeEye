<script lang="ts">
  interface Props {
    oncancel: () => void;
    onconfirm: (region: {
      x: number;
      y: number;
      width: number;
      height: number;
    }) => void;
  }

  let { onconfirm, oncancel }: Props = $props();

  // Selection state
  let isSelecting = $state(false);
  let selectionDone = $state(false);
  let startX = $state(0);
  let startY = $state(0);
  let endX = $state(0);
  let endY = $state(0);

  // Keyboard resize phase
  let resizeStep = 2;

  // Computed region (normalized so width/height are always positive)
  let region = $derived({
    x: Math.min(startX, endX),
    y: Math.min(startY, endY),
    width: Math.abs(endX - startX),
    height: Math.abs(endY - startY),
  });

  function handleMouseDown(e: MouseEvent) {
    if (selectionDone) {
      return;
    }
    isSelecting = true;
    startX = e.clientX;
    startY = e.clientY;
    endX = e.clientX;
    endY = e.clientY;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isSelecting) {
      return;
    }
    endX = e.clientX;
    endY = e.clientY;
  }

  function handleMouseUp(_e: MouseEvent) {
    if (!isSelecting) {
      return;
    }
    isSelecting = false;

    // Require minimum 10px selection
    if (region.width < 10 || region.height < 10) {
      startX = 0;
      startY = 0;
      endX = 0;
      endY = 0;
      return;
    }

    selectionDone = true;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      oncancel();
      return;
    }

    if (e.key === "Enter" && selectionDone) {
      e.preventDefault();
      onconfirm(region);
      return;
    }

    // Arrow key resize in the keyboard phase
    if (!selectionDone) {
      return;
    }

    const shift = e.shiftKey ? resizeStep * 5 : resizeStep;

    switch (e.key) {
      case "ArrowRight":
        e.preventDefault();
        endX += shift;
        break;
      case "ArrowLeft":
        e.preventDefault();
        endX -= shift;
        break;
      case "ArrowDown":
        e.preventDefault();
        endY += shift;
        break;
      case "ArrowUp":
        e.preventDefault();
        endY -= shift;
        break;
      default:
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- Fullscreen overlay -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-[9999] select-none"
  style:cursor={selectionDone ? "default" : "crosshair"}
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
>
  <!-- Dim background with cutout for selection -->
  {#if region.width > 0 && region.height > 0}
    <!-- Use SVG mask for the cutout effect -->
    <svg class="absolute inset-0 w-full h-full pointer-events-none" role="none">
      <defs>
        <mask id="overlay-mask">
          <rect x="0" y="0" width="100%" height="100%" fill="white" />
          <rect
            x={region.x}
            y={region.y}
            width={region.width}
            height={region.height}
            fill="black"
          />
        </mask>
      </defs>
      <rect
        x="0"
        y="0"
        width="100%"
        height="100%"
        fill="rgba(0, 0, 0, 0.5)"
        mask="url(#overlay-mask)"
      />
      <!-- Selection border -->
      <rect
        x={region.x}
        y={region.y}
        width={region.width}
        height={region.height}
        fill="none"
        stroke="#f97316"
        stroke-width="2"
        stroke-dasharray={selectionDone ? "0" : "6 3"}
      />
    </svg>

    <!-- Dimension label -->
    <div
      class="absolute text-xs font-mono px-2 py-0.5 rounded bg-[#f97316] text-black pointer-events-none"
      style:left="{region.x + region.width / 2}px"
      style:top="{region.y + region.height + 8}px"
      style:transform="translateX(-50%)"
    >
      {region.width}
      × {region.height}
    </div>

    <!-- Instructions -->
    {#if selectionDone}
      <div
        class="absolute text-xs font-mono px-3 py-1.5 rounded bg-black/80 text-white/80 pointer-events-none"
        style:left="{region.x + region.width / 2}px"
        style:top="{region.y - 36}px"
        style:transform="translateX(-50%)"
      >
        Enter to confirm · Arrow keys to resize · Esc to cancel
      </div>
    {/if}
  {:else}
    <!-- Full dim overlay before any selection -->
    <div class="absolute inset-0 bg-black/50 pointer-events-none"></div>
    <div
      class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2
        text-sm font-mono text-white/60 pointer-events-none"
    >
      Click and drag to select a region · Esc to cancel
    </div>
  {/if}

  <!-- Crosshair (only during selection) -->
  {#if isSelecting}
    <div
      class="absolute w-px bg-white/30 pointer-events-none"
      style:left="{endX}px"
      style:top="0"
      style:height="100vh"
    ></div>
    <div
      class="absolute h-px bg-white/30 pointer-events-none"
      style:top="{endY}px"
      style:left="0"
      style:width="100vw"
    ></div>
  {/if}
</div>
