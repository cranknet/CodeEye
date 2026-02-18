export interface Point {
  x: number;
  y: number;
}
export interface Rect {
  h: number;
  w: number;
  x: number;
  y: number;
}

/** Check if a point is inside a rectangle. */
export function pointInRect(p: Point, r: Rect): boolean {
  return p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;
}

/** Check if a point is inside an ellipse inscribed in bounds. */
export function pointInEllipse(p: Point, bounds: Rect): boolean {
  const cx = bounds.x + bounds.w / 2;
  const cy = bounds.y + bounds.h / 2;
  const rx = bounds.w / 2;
  const ry = bounds.h / 2;
  return (p.x - cx) ** 2 / rx ** 2 + (p.y - cy) ** 2 / ry ** 2 <= 1;
}

/** Check if a point is within `threshold` pixels of a line segment. */
export function pointNearLine(
  p: Point,
  a: Point,
  b: Point,
  threshold: number
): boolean {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const lenSq = dx * dx + dy * dy;
  if (lenSq === 0) {
    return Math.hypot(p.x - a.x, p.y - a.y) <= threshold;
  }
  const t = Math.max(
    0,
    Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / lenSq)
  );
  const projX = a.x + t * dx;
  const projY = a.y + t * dy;
  return Math.hypot(p.x - projX, p.y - projY) <= threshold;
}

/** Compute axis-aligned bounding box of a set of points. */
export function getBoundingBox(points: Point[]): Rect {
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const minX = Math.min(...xs);
  const minY = Math.min(...ys);
  return {
    x: minX,
    y: minY,
    w: Math.max(...xs) - minX,
    h: Math.max(...ys) - minY,
  };
}

/** Scale a rectangle around an origin point. */
export function scaleRect(r: Rect, factor: number, origin: Point): Rect {
  return {
    x: origin.x + (r.x - origin.x) * factor,
    y: origin.y + (r.y - origin.y) * factor,
    w: r.w * factor,
    h: r.h * factor,
  };
}

/** Convert absolute pixel position to percentage of image dimensions. */
export function percentagePosition(
  p: Point,
  imageWidth: number,
  imageHeight: number
): { percentX: number; percentY: number } {
  return {
    percentX: Math.round((p.x / imageWidth) * 100),
    percentY: Math.round((p.y / imageHeight) * 100),
  };
}
