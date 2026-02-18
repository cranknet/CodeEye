import type { Point } from "$lib/utils/geometry";

const MIN_ZOOM = 0.1;
const MAX_ZOOM = 10;

export function createCanvasStore() {
  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let imageWidth = $state(0);
  let imageHeight = $state(0);

  // Multi-frame support
  let frames = $state<string[]>([]);
  let currentFrame = $state(0);

  return {
    get zoom() {
      return zoom;
    },
    get panX() {
      return panX;
    },
    get panY() {
      return panY;
    },
    get imageWidth() {
      return imageWidth;
    },
    get imageHeight() {
      return imageHeight;
    },
    get frames() {
      return frames;
    },
    get currentFrame() {
      return currentFrame;
    },
    get frameCount() {
      return frames.length;
    },
    get currentImageSrc() {
      return frames[currentFrame] ?? null;
    },

    setImage(width: number, height: number) {
      imageWidth = width;
      imageHeight = height;
      zoom = 1;
      panX = 0;
      panY = 0;
    },

    /** Add a frame and switch to it. */
    addFrame(src: string) {
      frames = [...frames, src];
      currentFrame = frames.length - 1;
    },

    /** Replace all frames (e.g. when loading a single image). */
    setFrames(srcs: string[]) {
      frames = srcs;
      currentFrame = 0;
    },

    /** Switch to a specific frame index. */
    setFrame(index: number) {
      if (index >= 0 && index < frames.length) {
        currentFrame = index;
      }
    },

    /** Clear all frames. */
    clearFrames() {
      frames = [];
      currentFrame = 0;
    },

    zoomTo(level: number, cursorX: number, cursorY: number) {
      const oldZoom = zoom;
      zoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, level));
      const scale = zoom / oldZoom;
      panX = cursorX - (cursorX - panX) * scale;
      panY = cursorY - (cursorY - panY) * scale;
    },

    pan(dx: number, dy: number) {
      panX += dx;
      panY += dy;
    },

    resetView() {
      zoom = 1;
      panX = 0;
      panY = 0;
    },

    /** Convert screen coordinates to image coordinates. */
    screenToImage(sx: number, sy: number): Point {
      return {
        x: (sx - panX) / zoom,
        y: (sy - panY) / zoom,
      };
    },

    /** Convert image coordinates to screen coordinates. */
    imageToScreen(ix: number, iy: number): Point {
      return {
        x: ix * zoom + panX,
        y: iy * zoom + panY,
      };
    },
  };
}
