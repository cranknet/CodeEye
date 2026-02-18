import { describe, expect, it } from "vitest";
import {
  getBoundingBox,
  percentagePosition,
  pointInEllipse,
  pointInRect,
  pointNearLine,
  scaleRect,
} from "./geometry";

describe("pointInRect", () => {
  it("returns true for point inside rectangle", () => {
    expect(pointInRect({ x: 50, y: 50 }, { x: 0, y: 0, w: 100, h: 100 })).toBe(
      true
    );
  });

  it("returns false for point outside rectangle", () => {
    expect(pointInRect({ x: 150, y: 50 }, { x: 0, y: 0, w: 100, h: 100 })).toBe(
      false
    );
  });

  it("returns true for point on boundary", () => {
    expect(pointInRect({ x: 0, y: 0 }, { x: 0, y: 0, w: 100, h: 100 })).toBe(
      true
    );
  });
});

describe("pointInEllipse", () => {
  it("returns true for point at center", () => {
    expect(
      pointInEllipse({ x: 50, y: 50 }, { x: 0, y: 0, w: 100, h: 100 })
    ).toBe(true);
  });

  it("returns false for point at corner (outside ellipse)", () => {
    expect(pointInEllipse({ x: 5, y: 5 }, { x: 0, y: 0, w: 100, h: 100 })).toBe(
      false
    );
  });
});

describe("pointNearLine", () => {
  it("returns true for point near a horizontal line", () => {
    expect(
      pointNearLine({ x: 50, y: 52 }, { x: 0, y: 50 }, { x: 100, y: 50 }, 5)
    ).toBe(true);
  });

  it("returns false for point far from line", () => {
    expect(
      pointNearLine({ x: 50, y: 80 }, { x: 0, y: 50 }, { x: 100, y: 50 }, 5)
    ).toBe(false);
  });

  it("handles zero-length line (point)", () => {
    expect(
      pointNearLine({ x: 10, y: 11 }, { x: 10, y: 10 }, { x: 10, y: 10 }, 2)
    ).toBe(true);
  });
});

describe("getBoundingBox", () => {
  it("computes bounding box of points", () => {
    const box = getBoundingBox([
      { x: 10, y: 20 },
      { x: 50, y: 5 },
      { x: 30, y: 40 },
    ]);
    expect(box).toEqual({ x: 10, y: 5, w: 40, h: 35 });
  });
});

describe("scaleRect", () => {
  it("scales rectangle around origin", () => {
    const result = scaleRect({ x: 10, y: 10, w: 20, h: 20 }, 2, { x: 0, y: 0 });
    expect(result).toEqual({ x: 20, y: 20, w: 40, h: 40 });
  });
});

describe("percentagePosition", () => {
  it("calculates percentage correctly", () => {
    const pos = percentagePosition({ x: 384, y: 216 }, 1920, 1080);
    expect(pos.percentX).toBe(20);
    expect(pos.percentY).toBe(20);
  });

  it("handles edge positions", () => {
    const pos = percentagePosition({ x: 1920, y: 1080 }, 1920, 1080);
    expect(pos.percentX).toBe(100);
    expect(pos.percentY).toBe(100);
  });
});
