<script lang="ts">
  import { onMount } from 'svelte';
  import type { Point } from '$lib/utils/geometry';
  import type { createCanvasStore } from '$lib/state/canvas.svelte';
  import type { createAnnotationStore } from '$lib/state/annotations.svelte';
  import type { Annotation } from '$lib/state/annotations.svelte';
  import type { createToolStore } from '$lib/state/tools.svelte';

  type Props = {
    canvasState: ReturnType<typeof createCanvasStore>;
    annotationState: ReturnType<typeof createAnnotationStore>;
    toolState: ReturnType<typeof createToolStore>;
    imageSrc: string | null;
  };

  let { canvasState, annotationState, toolState, imageSrc }: Props = $props();

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
  let lastPanX = 0;
  let lastPanY = 0;

  // Selection state
  let selectedId: string | null = $state(null);

  // Severity color map
  const SEVERITY_COLORS: Record<string, string> = {
    critical: '#ef4444',
    minor: '#eab308',
    suggestion: '#3b82f6',
  };

  onMount(() => {
    ctx = canvasEl.getContext('2d')!;
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
      if (!ctx) return;
      const rect = canvasEl.getBoundingClientRect();
      const { width, height } = rect;
      canvasEl.width = width * devicePixelRatio;
      canvasEl.height = height * devicePixelRatio;
      ctx.scale(devicePixelRatio, devicePixelRatio);

      // Clear with dark background
      ctx.fillStyle = '#0a0a0a';
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

      // Draw active drawing preview
      drawPreview();

      animFrameId = requestAnimationFrame(render);
    }
    render();
  }

  // ─── Annotation Rendering ──────────────────────────────

  function drawAnnotations() {
    for (const ann of annotationState.annotations) {
      ctx.save();
      ctx.translate(canvasState.panX, canvasState.panY);
      ctx.scale(canvasState.zoom, canvasState.zoom);

      // Shadow for visibility on any background
      ctx.shadowColor = 'rgba(0, 0, 0, 0.5)';
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
      case 'rectangle':
        ctx.strokeRect(ann.bounds.x, ann.bounds.y, ann.bounds.w, ann.bounds.h);
        break;
      case 'circle':
        ctx.beginPath();
        ctx.ellipse(
          ann.bounds.x + ann.bounds.w / 2,
          ann.bounds.y + ann.bounds.h / 2,
          Math.abs(ann.bounds.w / 2),
          Math.abs(ann.bounds.h / 2),
          0,
          0,
          Math.PI * 2,
        );
        ctx.stroke();
        break;
      case 'arrow':
        if (ann.points && ann.points.length >= 2) {
          drawArrow(ann.points[0], ann.points[1]);
        }
        break;
      case 'freehand':
        if (ann.points && ann.points.length > 1) {
          ctx.beginPath();
          ctx.moveTo(ann.points[0].x, ann.points[0].y);
          for (let i = 1; i < ann.points.length; i++) {
            ctx.lineTo(ann.points[i].x, ann.points[i].y);
          }
          ctx.stroke();
        }
        break;
      case 'text':
        ctx.shadowBlur = 0;
        ctx.fillStyle = SEVERITY_COLORS[ann.severity] ?? ann.color;
        const fontSize = 16 / canvasState.zoom;
        ctx.font = `bold ${fontSize}px system-ui, sans-serif`;
        ctx.fillText(ann.label || 'Text', ann.bounds.x, ann.bounds.y + fontSize);
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
      to.y - headLen * Math.sin(angle - Math.PI / 6),
    );
    ctx.moveTo(to.x, to.y);
    ctx.lineTo(
      to.x - headLen * Math.cos(angle + Math.PI / 6),
      to.y - headLen * Math.sin(angle + Math.PI / 6),
    );
    ctx.stroke();
  }

  function drawNumberBadge(ann: Annotation, color: string) {
    const badgeSize = 18 / canvasState.zoom;
    const bx = ann.bounds.x - badgeSize / 2;
    const by = ann.bounds.y - badgeSize / 2;

    // Badge circle
    ctx.shadowBlur = 0;
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(bx + badgeSize / 2, by + badgeSize / 2, badgeSize / 2, 0, Math.PI * 2);
    ctx.fill();

    // Badge text
    ctx.fillStyle = '#ffffff';
    const fontSize = 11 / canvasState.zoom;
    ctx.font = `bold ${fontSize}px system-ui, sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(String(ann.number), bx + badgeSize / 2, by + badgeSize / 2);
    ctx.textAlign = 'start';
    ctx.textBaseline = 'alphabetic';
  }

  // ─── Drawing Preview ──────────────────────────────────

  function drawPreview() {
    if (!isDrawing || !drawStart) return;

    ctx.save();
    ctx.translate(canvasState.panX, canvasState.panY);
    ctx.scale(canvasState.zoom, canvasState.zoom);

    ctx.strokeStyle = SEVERITY_COLORS[toolState.activeSeverity] ?? toolState.activeColor;
    ctx.lineWidth = 2 / canvasState.zoom;
    ctx.setLineDash([4 / canvasState.zoom, 4 / canvasState.zoom]);
    ctx.globalAlpha = 0.7;

    const tool = toolState.activeTool;

    if (tool === 'rectangle' && drawEnd) {
      const x = Math.min(drawStart.x, drawEnd.x);
      const y = Math.min(drawStart.y, drawEnd.y);
      const w = Math.abs(drawEnd.x - drawStart.x);
      const h = Math.abs(drawEnd.y - drawStart.y);
      ctx.strokeRect(x, y, w, h);
    } else if (tool === 'circle' && drawEnd) {
      const cx = (drawStart.x + drawEnd.x) / 2;
      const cy = (drawStart.y + drawEnd.y) / 2;
      const rx = Math.abs(drawEnd.x - drawStart.x) / 2;
      const ry = Math.abs(drawEnd.y - drawStart.y) / 2;
      ctx.beginPath();
      ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
      ctx.stroke();
    } else if (tool === 'arrow' && drawEnd) {
      drawArrow(drawStart, drawEnd);
    } else if (tool === 'freehand' && freehandPoints.length > 1) {
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
      e.clientY - rect.top,
    );
  }

  function handleMouseDown(e: MouseEvent) {
    // Middle click → pan
    if (e.button === 1) {
      isPanning = true;
      lastPanX = e.clientX;
      lastPanY = e.clientY;
      return;
    }

    if (e.button !== 0) return;

    const rect = canvasEl.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const imgPos = canvasState.screenToImage(screenX, screenY);

    const tool = toolState.activeTool;

    if (tool === 'select') {
      // Hit-test annotations (reverse order, topmost first)
      let hit: Annotation | null = null;
      for (let i = annotationState.annotations.length - 1; i >= 0; i--) {
        const ann = annotationState.annotations[i];
        if (hitTestAnnotation(imgPos, ann)) {
          hit = ann;
          break;
        }
      }
      selectedId = hit?.id ?? null;
      return;
    }

    // Start drawing
    isDrawing = true;
    drawStart = imgPos;
    drawEnd = imgPos;

    if (tool === 'freehand') {
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

    if (!isDrawing || !drawStart) return;

    const rect = canvasEl.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const imgPos = canvasState.screenToImage(screenX, screenY);

    drawEnd = imgPos;

    if (toolState.activeTool === 'freehand') {
      freehandPoints = [...freehandPoints, imgPos];
    }
  }

  function handleMouseUp(e: MouseEvent) {
    if (isPanning) {
      isPanning = false;
      return;
    }

    if (!isDrawing || !drawStart || !drawEnd) {
      isDrawing = false;
      return;
    }

    const tool = toolState.activeTool;

    if (tool === 'rectangle' || tool === 'circle') {
      const x = Math.min(drawStart.x, drawEnd.x);
      const y = Math.min(drawStart.y, drawEnd.y);
      const w = Math.abs(drawEnd.x - drawStart.x);
      const h = Math.abs(drawEnd.y - drawStart.y);

      // Ignore tiny accidental draws
      if (w > 3 && h > 3) {
        annotationState.add({
          type: tool,
          bounds: { x, y, w, h },
          label: toolState.activeQuickLabel ?? '',
          severity: toolState.activeSeverity,
          color: toolState.activeColor,
        });
      }
    } else if (tool === 'arrow') {
      const dx = drawEnd.x - drawStart.x;
      const dy = drawEnd.y - drawStart.y;
      if (Math.hypot(dx, dy) > 5) {
        const x = Math.min(drawStart.x, drawEnd.x);
        const y = Math.min(drawStart.y, drawEnd.y);
        const w = Math.abs(dx);
        const h = Math.abs(dy);
        annotationState.add({
          type: 'arrow',
          bounds: { x, y, w, h },
          points: [{ ...drawStart }, { ...drawEnd }],
          label: toolState.activeQuickLabel ?? '',
          severity: toolState.activeSeverity,
          color: toolState.activeColor,
        });
      }
    } else if (tool === 'freehand' && freehandPoints.length > 2) {
      const xs = freehandPoints.map((p) => p.x);
      const ys = freehandPoints.map((p) => p.y);
      const minX = Math.min(...xs);
      const minY = Math.min(...ys);
      annotationState.add({
        type: 'freehand',
        bounds: {
          x: minX,
          y: minY,
          w: Math.max(...xs) - minX,
          h: Math.max(...ys) - minY,
        },
        points: [...freehandPoints],
        label: toolState.activeQuickLabel ?? '',
        severity: toolState.activeSeverity,
        color: toolState.activeColor,
      });
    } else if (tool === 'text') {
      annotationState.add({
        type: 'text',
        bounds: { x: drawStart.x, y: drawStart.y, w: 100, h: 20 },
        label: toolState.activeQuickLabel || 'Text',
        severity: toolState.activeSeverity,
        color: toolState.activeColor,
      });
    }

    // Reset drawing state
    isDrawing = false;
    drawStart = null;
    drawEnd = null;
    freehandPoints = [];
  }

  // ─── Hit Testing ──────────────────────────────────

  function hitTestAnnotation(p: Point, ann: Annotation): boolean {
    const { x, y, w, h } = ann.bounds;
    // Simple bounding box check for all types
    return p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h;
  }

  // ─── Keyboard Shortcuts ──────────────────────────────

  function handleKeyDown(e: KeyboardEvent) {
    // Undo/Redo
    if ((e.metaKey || e.ctrlKey) && e.key === 'z') {
      e.preventDefault();
      if (e.shiftKey) {
        annotationState.redo();
      } else {
        annotationState.undo();
      }
      return;
    }

    // Delete selected
    if ((e.key === 'Delete' || e.key === 'Backspace') && selectedId) {
      annotationState.remove(selectedId);
      selectedId = null;
      return;
    }

    // Tool shortcuts
    const toolMap: Record<string, typeof toolState.activeTool> = {
      v: 'select',
      s: 'rectangle',
      c: 'circle',
      a: 'arrow',
      f: 'freehand',
      t: 'text',
    };

    if (toolMap[e.key] && !e.metaKey && !e.ctrlKey && !e.altKey) {
      toolState.setTool(toolMap[e.key]);
    }
  }

  // Derive cursor from active tool
  let cursor = $derived.by(() => {
    switch (toolState.activeTool) {
      case 'select':
        return 'default';
      case 'text':
        return 'text';
      default:
        return 'crosshair';
    }
  });
</script>

<svelte:window onkeydown={handleKeyDown} />

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
