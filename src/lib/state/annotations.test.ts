import { describe, expect, it } from "vitest";
import { createAnnotationStore } from "./annotations.svelte";

describe("annotation store", () => {
  it("adds an annotation", () => {
    const store = createAnnotationStore();
    store.add({
      type: "rectangle",
      bounds: { x: 10, y: 10, w: 100, h: 50 },
      label: "spacing",
      severity: "minor",
    });
    expect(store.annotations.length).toBe(1);
    expect(store.annotations[0].number).toBe(1);
    expect(store.annotations[0].type).toBe("rectangle");
  });

  it("auto-increments annotation numbers", () => {
    const store = createAnnotationStore();
    store.add({
      type: "circle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "suggestion",
    });
    store.add({
      type: "arrow",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "critical",
    });
    expect(store.annotations[0].number).toBe(1);
    expect(store.annotations[1].number).toBe(2);
  });

  it("removes an annotation", () => {
    const store = createAnnotationStore();
    store.add({
      type: "rectangle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "minor",
    });
    const id = store.annotations[0].id;
    store.remove(id);
    expect(store.annotations.length).toBe(0);
  });

  it("updates an annotation", () => {
    const store = createAnnotationStore();
    store.add({
      type: "rectangle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "old",
      severity: "minor",
    });
    const id = store.annotations[0].id;
    store.update(id, { label: "new", severity: "critical" });
    expect(store.annotations[0].label).toBe("new");
    expect(store.annotations[0].severity).toBe("critical");
  });

  it("supports undo", () => {
    const store = createAnnotationStore();
    store.add({
      type: "rectangle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "minor",
    });
    expect(store.annotations.length).toBe(1);
    store.undo();
    expect(store.annotations.length).toBe(0);
  });

  it("supports redo", () => {
    const store = createAnnotationStore();
    store.add({
      type: "rectangle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "minor",
    });
    store.undo();
    expect(store.annotations.length).toBe(0);
    store.redo();
    expect(store.annotations.length).toBe(1);
  });

  it("canUndo/canRedo report correctly", () => {
    const store = createAnnotationStore();
    expect(store.canUndo).toBe(false);
    expect(store.canRedo).toBe(false);

    store.add({
      type: "circle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "minor",
    });
    expect(store.canUndo).toBe(true);
    expect(store.canRedo).toBe(false);

    store.undo();
    expect(store.canUndo).toBe(false);
    expect(store.canRedo).toBe(true);
  });

  it("clear resets everything", () => {
    const store = createAnnotationStore();
    store.add({
      type: "rectangle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "minor",
    });
    store.add({
      type: "circle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "minor",
    });
    store.clear();
    expect(store.annotations.length).toBe(0);
    expect(store.canUndo).toBe(false);
  });

  it("loadAnnotations restores state", () => {
    const store = createAnnotationStore();
    const data = [
      {
        id: "test-1",
        type: "rectangle" as const,
        number: 5,
        frame: 0,
        bounds: { x: 0, y: 0, w: 100, h: 100 },
        label: "loaded",
        comment: "",
        severity: "critical" as const,
        color: "#ff0000",
        created_at: 1000,
      },
    ];
    store.loadAnnotations(data);
    expect(store.annotations.length).toBe(1);
    expect(store.annotations[0].label).toBe("loaded");

    // Next added annotation should continue numbering
    store.add({
      type: "circle",
      bounds: { x: 0, y: 0, w: 50, h: 50 },
      label: "",
      severity: "minor",
    });
    expect(store.annotations[1].number).toBe(6);
  });
});
