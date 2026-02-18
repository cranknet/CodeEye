<script lang="ts">
  import { onMount } from "svelte";
  import type {
    Annotation,
    createAnnotationStore,
  } from "$lib/state/annotations.svelte";
  import type { createCanvasStore } from "$lib/state/canvas.svelte";
  import type { createToolStore } from "$lib/state/tools.svelte";
  import type { Point, Rect } from "$lib/utils/geometry";
  import { pointInRect } from "$lib/utils/geometry";

  interface Props {
    annotationState: ReturnType<typeof createAnnotationStore>;
    canvasBg?: string;
    canvasState: ReturnType<typeof createCanvasStore>;
    imageSrc: string | null;
    onselect?: (id: string | null) => void;
    selectedId?: string | null;
    toolState: ReturnType<typeof createToolStore>;
  }

  let {
    canvasState,
    annotationState,
    toolState,
    imageSrc,
    canvasBg = "#e8e8e8",
    selectedId = $bindable(null),
    onselect,
  }: Props = $props();

  // Filter annotations by current frame for rendering and hit-testing
  let visibleAnnotations = $derived(
    canvasState.frameCount > 1
      ? annotationState.annotations.filter(
          (a) => a.frame === canvasState.currentFrame
        )
      : annotationState.annotations
  );

  let canvasEl: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let image: HTMLImageElement | null = $state(null);
  let animFrameId: number;

  // Drawing state for active annotation creation
  let isDrawing = $state(false);
  let drawStart: Point | null = $state(null);
  let drawEnd: Point | null = $state(null);
  let freehandPoints: Point[] = $state([]);

  // Panning state
  let isPanning = $state(false);
  let spaceHeld = $state(false);
  let lastPanX = 0;
  let lastPanY = 0;

  // Select/move/resize state
  type DragMode = "move" | "resize" | null;
  let dragMode: DragMode = $state(null);
  let dragHandleIndex = $state(-1); // which of the 8 handles
  let dragStartImg: Point | null = $state(null);
  let dragOrigBounds: Rect | null = $state(null);
  let dragOrigPoints: Point[] | null = $state(null);

  // Resize handle size in screen pixels (constant regardless of zoom)
  const HANDLE_SCREEN_SIZE = 8;

  // Severity color map
  const SEVERITY_COLORS: Record<string, string> = {
    critical: "#ef4444",
    minor: "#eab308",
    suggestion: "#3b82f6",
  };

  onMount(() => {
    const context = canvasEl.getContext("2d");
    if (!context) {
      throw new Error("Canvas 2D context not available");
    }
    ctx = context;
    startRenderLoop();
    return () => cancelAnimationFrame(animFrameId);
  });

  $effect(() => {
    if (imageSrc) {
      const img = new Image();
      img.onload = () => {
        image = img;
        canvasState.setImage(img.width, img.height);
      };
      img.src = imageSrc;
    }
  });

  function startRenderLoop() {
    function render() {
      if (!ctx) {
        return;
      }
      const rect = canvasEl.getBoundingClientRect();
      const { width, height } = rect;
      canvasEl.width = width * devicePixelRatio;
      canvasEl.height = height * devicePixelRatio;
      ctx.scale(devicePixelRatio, devicePixelRatio);
      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = "high";

      // Clear with theme-aware background
      ctx.fillStyle = canvasBg;
      ctx.fillRect(0, 0, width, height);

      // Draw image with zoom/pan transform
      if (image) {
        ctx.save();
        ctx.translate(canvasState.panX, canvasState.panY);
        ctx.scale(canvasState.zoom, canvasState.zoom);
        ctx.drawImage(image, 0, 0);
        ctx.restore();
      }

      // Draw annotations
      drawAnnotations();

      // Draw resize handles for selected
      if (selectedId) {
        const sel = visibleAnnotations.find((a) => a.id === selectedId);
        if (sel) {
          drawResizeHandles(sel);
        }
      }

      // Draw active drawing preview
      drawPreview();

      animFrameId = requestAnimationFrame(render);
    }
    render();
  }

  // ─── Resize Handle Positions ──────────────────────────

  /** Returns 8 handle positions (image coords) for an annotation's bounds:
   * 0=TL, 1=TC, 2=TR, 3=ML, 4=MR, 5=BL, 6=BC, 7=BR */
  function getHandlePositions(b: Rect): Point[] {
    return [
      { x: b.x, y: b.y }, // 0: top-left
      { x: b.x + b.w / 2, y: b.y }, // 1: top-center
      { x: b.x + b.w, y: b.y }, // 2: top-right
      { x: b.x, y: b.y + b.h / 2 }, // 3: mid-left
      { x: b.x + b.w, y: b.y + b.h / 2 }, // 4: mid-right
      { x: b.x, y: b.y + b.h }, // 5: bottom-left
      { x: b.x + b.w / 2, y: b.y + b.h }, // 6: bottom-center
      { x: b.x + b.w, y: b.y + b.h }, // 7: bottom-right
    ];
  }

  /** Cursor style for each resize handle index */
  const HANDLE_CURSORS = [
    "nwse-resize",
    "ns-resize",
    "nesw-resize",
    "ew-resize",
    "ew-resize",
    "nesw-resize",
    "ns-resize",
    "nwse-resize",
  ];

  function drawResizeHandles(ann: Annotation) {
    const handles = getHandlePositions(ann.bounds);
    const size = HANDLE_SCREEN_SIZE; // screen pixels

    for (const hp of handles) {
      const screen = canvasState.imageToScreen(hp.x, hp.y);
      ctx.fillStyle = "#ffffff";
      ctx.strokeStyle = "#f97316";
      ctx.lineWidth = 1.5;
      ctx.fillRect(screen.x - size / 2, screen.y - size / 2, size, size);
      ctx.strokeRect(screen.x - size / 2, screen.y - size / 2, size, size);
    }
  }

  /** Hit-test handles. Returns handle index or -1. */
  function hitTestHandles(screenPos: Point, ann: Annotation): number {
    const handles = getHandlePositions(ann.bounds);
    const halfSize = (HANDLE_SCREEN_SIZE + 4) / 2; // extra tolerance

    for (let i = 0; i < handles.length; i++) {
      const screen = canvasState.imageToScreen(handles[i].x, handles[i].y);
      if (
        Math.abs(screenPos.x - screen.x) <= halfSize &&
        Math.abs(screenPos.y - screen.y) <= halfSize
      ) {
        return i;
      }
    }
    return -1;
  }

  // ─── Annotation Rendering ──────────────────────────────

  function drawAnnotations() {
    for (const ann of visibleAnnotations) {
      ctx.save();
      ctx.translate(canvasState.panX, canvasState.panY);
      ctx.scale(canvasState.zoom, canvasState.zoom);

      // Shadow for visibility on any background
      ctx.shadowColor = "rgba(0, 0, 0, 0.5)";
      ctx.shadowBlur = 4;
      ctx.shadowOffsetX = 1;
      ctx.shadowOffsetY = 1;

      const color = SEVERITY_COLORS[ann.severity] ?? ann.color;
      ctx.strokeStyle = color;
      ctx.lineWidth = 2 / canvasState.zoom;

      const isSelected = ann.id === selectedId;
      if (isSelected) {
        ctx.lineWidth = 3 / canvasState.zoom;
        ctx.setLineDash([6 / canvasState.zoom, 3 / canvasState.zoom]);
      }

      drawAnnotationShape(ann);

      ctx.setLineDash([]);

      // Number badge
      drawNumberBadge(ann, color);

      ctx.restore();
    }
  }

  function drawAnnotationShape(ann: Annotation) {
    switch (ann.type) {
      case "rectangle":
        ctx.strokeRect(ann.bounds.x, ann.bounds.y, ann.bounds.w, ann.bounds.h);
        break;
      case "circle":
        ctx.beginPath();
        ctx.ellipse(
          ann.bounds.x + ann.bounds.w / 2,
          ann.bounds.y + ann.bounds.h / 2,
          Math.abs(ann.bounds.w / 2),
          Math.abs(ann.bounds.h / 2),
          0,
          0,
          Math.PI * 2
        );
        ctx.stroke();
        break;
      case "arrow":
        if (ann.points && ann.points.length >= 2) {
          drawArrow(ann.points[0], ann.points[1]);
        }
        break;
      case "freehand":
        if (ann.points && ann.points.length > 1) {
          ctx.beginPath();
          ctx.moveTo(ann.points[0].x, ann.points[0].y);
          for (let i = 1; i < ann.points.length; i++) {
            ctx.lineTo(ann.points[i].x, ann.points[i].y);
          }
          ctx.stroke();
        }
        break;
      case "text": {
        ctx.shadowBlur = 0;
        ctx.fillStyle = SEVERITY_COLORS[ann.severity] ?? ann.color;
        const fontSize = 16 / canvasState.zoom;
        ctx.font = `bold ${fontSize}px system-ui, sans-serif`;
        ctx.fillText(
          ann.label || "Text",
          ann.bounds.x,
          ann.bounds.y + fontSize
        );
        break;
      }
      default:
        break;
    }
  }

  function drawArrow(from: Point, to: Point) {
    const headLen = 12 / canvasState.zoom;
    const angle = Math.atan2(to.y - from.y, to.x - from.x);

    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.lineTo(to.x, to.y);
    ctx.stroke();

    // Arrowhead
    ctx.beginPath();
    ctx.moveTo(to.x, to.y);
    ctx.lineTo(
      to.x - headLen * Math.cos(angle - Math.PI / 6),
      to.y - headLen * Math.sin(angle - Math.PI / 6)
    );
    ctx.moveTo(to.x, to.y);
    ctx.lineTo(
      to.x - headLen * Math.cos(angle + Math.PI / 6),
      to.y - headLen * Math.sin(angle + Math.PI / 6)
    );
    ctx.stroke();
  }

  function drawNumberBadge(ann: Annotation, color: string) {
    const badgeSize = 18 / canvasState.zoom;
    const bx = ann.bounds.x - badgeSize / 2;
    const by = ann.bounds.y - badgeSize / 2;

    ctx.shadowBlur = 0;
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(
      bx + badgeSize / 2,
      by + badgeSize / 2,
      badgeSize / 2,
      0,
      Math.PI * 2
    );
    ctx.fill();

    ctx.fillStyle = "#ffffff";
    const fontSize = 11 / canvasState.zoom;
    ctx.font = `bold ${fontSize}px system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(String(ann.number), bx + badgeSize / 2, by + badgeSize / 2);
    ctx.textAlign = "start";
    ctx.textBaseline = "alphabetic";
  }

  // ─── Drawing Preview ──────────────────────────────────

  function drawPreview() {
    if (!(isDrawing && drawStart)) {
      return;
    }

    ctx.save();
    ctx.translate(canvasState.panX, canvasState.panY);
    ctx.scale(canvasState.zoom, canvasState.zoom);

    ctx.strokeStyle =
      SEVERITY_COLORS[toolState.activeSeverity] ?? toolState.activeColor;
    ctx.lineWidth = 2 / canvasState.zoom;
    ctx.setLineDash([4 / canvasState.zoom, 4 / canvasState.zoom]);
    ctx.globalAlpha = 0.7;

    const tool = toolState.activeTool;

    if (tool === "rectangle" && drawEnd) {
      const x = Math.min(drawStart.x, drawEnd.x);
      const y = Math.min(drawStart.y, drawEnd.y);
      const w = Math.abs(drawEnd.x - drawStart.x);
      const h = Math.abs(drawEnd.y - drawStart.y);
      ctx.strokeRect(x, y, w, h);
    } else if (tool === "circle" && drawEnd) {
      const cx = (drawStart.x + drawEnd.x) / 2;
      const cy = (drawStart.y + drawEnd.y) / 2;
      const rx = Math.abs(drawEnd.x - drawStart.x) / 2;
      const ry = Math.abs(drawEnd.y - drawStart.y) / 2;
      ctx.beginPath();
      ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
      ctx.stroke();
    } else if (tool === "arrow" && drawEnd) {
      drawArrow(drawStart, drawEnd);
    } else if (tool === "freehand" && freehandPoints.length > 1) {
      ctx.setLineDash([]);
      ctx.beginPath();
      ctx.moveTo(freehandPoints[0].x, freehandPoints[0].y);
      for (let i = 1; i < freehandPoints.length; i++) {
        ctx.lineTo(freehandPoints[i].x, freehandPoints[i].y);
      }
      ctx.stroke();
    }

    ctx.restore();
  }

  // ─── Mouse Handlers ──────────────────────────────────

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const rect = canvasEl.getBoundingClientRect();
    canvasState.zoomTo(
      canvasState.zoom * delta,
      e.clientX - rect.left,
      e.clientY - rect.top
    );
  }

  /** Handle select-tool mouse down: resize handles, hit-test, or fallback pan */
  function handleSelectDown(
    screenPos: Point,
    imgPos: Point,
    clientX: number,
    clientY: number
  ) {
    // Priority 1: check resize handles on selected annotation
    if (selectedId) {
      const sel = visibleAnnotations.find((a) => a.id === selectedId);
      if (sel) {
        const hIdx = hitTestHandles(screenPos, sel);
        if (hIdx >= 0) {
          dragMode = "resize";
          dragHandleIndex = hIdx;
          dragStartImg = imgPos;
          dragOrigBounds = { ...sel.bounds };
          dragOrigPoints = sel.points
            ? sel.points.map((p) => ({ ...p }))
            : null;
          return;
        }
      }
    }

    // Priority 2: hit-test annotation bodies (reverse order, topmost first)
    const hit = findTopmostAnnotation(imgPos);

    if (hit) {
      selectedId = hit.id;
      onselect?.(hit.id);
      dragMode = "move";
      dragStartImg = imgPos;
      dragOrigBounds = { ...hit.bounds };
      dragOrigPoints = hit.points ? hit.points.map((p) => ({ ...p })) : null;
    } else {
      // No annotation hit — deselect and start fallback pan
      selectedId = null;
      onselect?.(null);
      isPanning = true;
      lastPanX = clientX;
      lastPanY = clientY;
    }
  }

  /** Find the topmost annotation at a given image-space point */
  function findTopmostAnnotation(imgPos: Point): Annotation | null {
    for (let i = visibleAnnotations.length - 1; i >= 0; i--) {
      const ann = visibleAnnotations[i];
      if (hitTestAnnotation(imgPos, ann)) {
        return ann;
      }
    }
    return null;
  }

  function handleMouseDown(e: MouseEvent) {
    // Middle click → pan
    if (e.button === 1) {
      isPanning = true;
      lastPanX = e.clientX;
      lastPanY = e.clientY;
      return;
    }

    if (e.button !== 0) {
      return;
    }

    // Spacebar+left-click → pan (works in any tool mode)
    if (spaceHeld) {
      isPanning = true;
      lastPanX = e.clientX;
      lastPanY = e.clientY;
      return;
    }

    const rect = canvasEl.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const screenPos: Point = { x: screenX, y: screenY };
    const imgPos = canvasState.screenToImage(screenX, screenY);

    if (toolState.activeTool === "select") {
      handleSelectDown(screenPos, imgPos, e.clientX, e.clientY);
      return;
    }

    // Start drawing
    isDrawing = true;
    drawStart = imgPos;
    drawEnd = imgPos;

    if (toolState.activeTool === "freehand") {
      freehandPoints = [imgPos];
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (isPanning) {
      canvasState.pan(e.clientX - lastPanX, e.clientY - lastPanY);
      lastPanX = e.clientX;
      lastPanY = e.clientY;
      return;
    }

    // Move or resize selected annotation
    if (dragMode && selectedId && dragStartImg && dragOrigBounds) {
      const rect = canvasEl.getBoundingClientRect();
      const imgPos = canvasState.screenToImage(
        e.clientX - rect.left,
        e.clientY - rect.top
      );
      const dx = imgPos.x - dragStartImg.x;
      const dy = imgPos.y - dragStartImg.y;

      if (dragMode === "move") {
        const newBounds: Rect = {
          x: dragOrigBounds.x + dx,
          y: dragOrigBounds.y + dy,
          w: dragOrigBounds.w,
          h: dragOrigBounds.h,
        };
        const changes: Partial<Annotation> = { bounds: newBounds };
        if (dragOrigPoints) {
          changes.points = dragOrigPoints.map((p) => ({
            x: p.x + dx,
            y: p.y + dy,
          }));
        }
        annotationState.update(selectedId, changes);
      } else if (dragMode === "resize") {
        const newBounds = computeResize(
          dragOrigBounds,
          dragHandleIndex,
          dx,
          dy,
          e.shiftKey
        );
        annotationState.update(selectedId, { bounds: newBounds });
      }
      return;
    }

    if (!(isDrawing && drawStart)) {
      return;
    }

    const rect = canvasEl.getBoundingClientRect();
    const imgPos = canvasState.screenToImage(
      e.clientX - rect.left,
      e.clientY - rect.top
    );

    drawEnd = imgPos;

    if (toolState.activeTool === "freehand") {
      freehandPoints = [...freehandPoints, imgPos];
    }
  }

  function handleMouseUp(_e: MouseEvent) {
    if (isPanning) {
      isPanning = false;
      return;
    }

    if (dragMode) {
      dragMode = null;
      dragHandleIndex = -1;
      dragStartImg = null;
      dragOrigBounds = null;
      dragOrigPoints = null;
      return;
    }

    if (!(isDrawing && drawStart && drawEnd)) {
      isDrawing = false;
      return;
    }

    const tool = toolState.activeTool;

    if (tool === "rectangle" || tool === "circle") {
      const x = Math.min(drawStart.x, drawEnd.x);
      const y = Math.min(drawStart.y, drawEnd.y);
      const w = Math.abs(drawEnd.x - drawStart.x);
      const h = Math.abs(drawEnd.y - drawStart.y);

      if (w > 3 && h > 3) {
        annotationState.add({
          type: tool,
          bounds: { x, y, w, h },
          label: toolState.activeQuickLabel ?? "",
          severity: toolState.activeSeverity,
          color: toolState.activeColor,
          frame: canvasState.currentFrame,
        });
      }
    } else if (tool === "arrow") {
      const dist = Math.hypot(drawEnd.x - drawStart.x, drawEnd.y - drawStart.y);
      if (dist > 5) {
        const x = Math.min(drawStart.x, drawEnd.x);
        const y = Math.min(drawStart.y, drawEnd.y);
        annotationState.add({
          type: "arrow",
          bounds: {
            x,
            y,
            w: Math.abs(drawEnd.x - drawStart.x),
            h: Math.abs(drawEnd.y - drawStart.y),
          },
          points: [{ ...drawStart }, { ...drawEnd }],
          label: toolState.activeQuickLabel ?? "",
          severity: toolState.activeSeverity,
          color: toolState.activeColor,
          frame: canvasState.currentFrame,
        });
      }
    } else if (tool === "freehand" && freehandPoints.length > 2) {
      const xs = freehandPoints.map((p) => p.x);
      const ys = freehandPoints.map((p) => p.y);
      const minX = Math.min(...xs);
      const minY = Math.min(...ys);
      annotationState.add({
        type: "freehand",
        bounds: {
          x: minX,
          y: minY,
          w: Math.max(...xs) - minX,
          h: Math.max(...ys) - minY,
        },
        points: [...freehandPoints],
        label: toolState.activeQuickLabel ?? "",
        severity: toolState.activeSeverity,
        color: toolState.activeColor,
        frame: canvasState.currentFrame,
      });
    } else if (tool === "text") {
      annotationState.add({
        type: "text",
        bounds: { x: drawStart.x, y: drawStart.y, w: 100, h: 20 },
        label: toolState.activeQuickLabel || "Text",
        severity: toolState.activeSeverity,
        color: toolState.activeColor,
        frame: canvasState.currentFrame,
      });
    }

    isDrawing = false;
    drawStart = null;
    drawEnd = null;
    freehandPoints = [];
  }

  // ─── Resize Computation ──────────────────────────────

  /** Compute new bounds after dragging a resize handle.
   * Handle indices: 0=TL, 1=TC, 2=TR, 3=ML, 4=MR, 5=BL, 6=BC, 7=BR */
  function computeResize(
    orig: Rect,
    handle: number,
    dx: number,
    dy: number,
    lockAspect: boolean
  ): Rect {
    let { x, y, w, h } = orig;

    // Which edges move
    const movesLeft = handle === 0 || handle === 3 || handle === 5;
    const movesRight = handle === 2 || handle === 4 || handle === 7;
    const movesTop = handle === 0 || handle === 1 || handle === 2;
    const movesBottom = handle === 5 || handle === 6 || handle === 7;

    if (movesLeft) {
      x += dx;
      w -= dx;
    }
    if (movesRight) {
      w += dx;
    }
    if (movesTop) {
      y += dy;
      h -= dy;
    }
    if (movesBottom) {
      h += dy;
    }

    // Enforce minimum size
    if (w < 5) {
      w = 5;
    }
    if (h < 5) {
      h = 5;
    }

    // Aspect ratio lock with shift
    if (lockAspect && orig.w > 0 && orig.h > 0) {
      const ratio = orig.w / orig.h;
      if (movesRight || movesLeft) {
        h = w / ratio;
      } else {
        w = h * ratio;
      }
    }

    return { x, y, w, h };
  }

  // ─── Hit Testing ──────────────────────────────────

  function hitTestAnnotation(p: Point, ann: Annotation): boolean {
    const { x, y, w, h } = ann.bounds;
    return pointInRect(p, { x, y, w, h });
  }

  // ─── Keyboard Shortcuts ──────────────────────────────

  function isTextInput(e: KeyboardEvent): boolean {
    const target = e.target as HTMLElement;
    return target.tagName === "INPUT" || target.tagName === "TEXTAREA";
  }

  const TOOL_KEYS: Record<string, typeof toolState.activeTool> = {
    v: "select",
    s: "rectangle",
    c: "circle",
    a: "arrow",
    f: "freehand",
    t: "text",
  };

  function handleToolShortcut(e: KeyboardEvent) {
    if (isTextInput(e) || e.metaKey || e.ctrlKey || e.altKey) {
      return;
    }
    if (TOOL_KEYS[e.key]) {
      toolState.setTool(TOOL_KEYS[e.key]);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Spacebar hold for pan mode
    if (e.code === "Space" && !e.repeat && !isTextInput(e)) {
      e.preventDefault();
      spaceHeld = true;
      return;
    }

    // Undo/Redo
    if ((e.metaKey || e.ctrlKey) && e.key === "z") {
      e.preventDefault();
      if (e.shiftKey) {
        annotationState.redo();
      } else {
        annotationState.undo();
      }
      return;
    }

    // Delete selected
    if ((e.key === "Delete" || e.key === "Backspace") && selectedId) {
      annotationState.remove(selectedId);
      selectedId = null;
      onselect?.(null);
      return;
    }

    // Reset view
    if ((e.metaKey || e.ctrlKey) && e.key === "0") {
      e.preventDefault();
      canvasState.resetView();
      return;
    }

    // Tool shortcuts
    handleToolShortcut(e);
  }

  function handleKeyUp(e: KeyboardEvent) {
    if (e.code === "Space") {
      spaceHeld = false;
    }
  }

  // Derive cursor from context
  let cursor = $derived.by(() => {
    if (isPanning) {
      return "grabbing";
    }
    if (spaceHeld) {
      return "grab";
    }
    if (dragMode === "move") {
      return "move";
    }
    if (dragMode === "resize") {
      return HANDLE_CURSORS[dragHandleIndex] ?? "nwse-resize";
    }
    switch (toolState.activeTool) {
      case "select":
        return "default";
      case "text":
        return "text";
      default:
        return "crosshair";
    }
  });
</script>

<svelte:window onkeydown={handleKeyDown} onkeyup={handleKeyUp} />

<canvas
  bind:this={canvasEl}
  class="w-full h-full outline-none"
  style:cursor
  tabindex="0"
  onwheel={handleWheel}
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  onmouseleave={handleMouseUp}
></canvas>
