import type { Point, Rect } from "$lib/utils/geometry";

export type AnnotationType =
  | "circle"
  | "rectangle"
  | "arrow"
  | "freehand"
  | "text";
export type Severity = "critical" | "minor" | "suggestion";

export interface Annotation {
  bounds: Rect;
  color: string;
  comment: string;
  created_at: number;
  frame: number;
  id: string;
  label: string;
  number: number;
  points?: Point[];
  severity: Severity;
  type: AnnotationType;
}

interface AddInput {
  bounds: Rect;
  color?: string;
  comment?: string;
  frame?: number;
  label: string;
  points?: Point[];
  severity: Severity;
  type: AnnotationType;
}

export function createAnnotationStore() {
  let annotations = $state<Annotation[]>([]);
  let undoStack = $state<Annotation[][]>([[]]);
  let undoIndex = $state(0);
  let nextNumber = $state(1);

  function snapshot() {
    undoStack = undoStack.slice(0, undoIndex + 1);
    undoStack.push(structuredClone($state.snapshot(annotations)));
    undoIndex = undoStack.length - 1;
  }

  return {
    get annotations() {
      return annotations;
    },
    get canUndo() {
      return undoIndex > 0;
    },
    get canRedo() {
      return undoIndex < undoStack.length - 1;
    },

    add(input: AddInput) {
      const annotation: Annotation = {
        id: crypto.randomUUID(),
        type: input.type,
        number: nextNumber++,
        frame: input.frame ?? 0,
        bounds: input.bounds,
        points: input.points,
        label: input.label,
        comment: input.comment ?? "",
        severity: input.severity,
        color: input.color ?? "#f97316",
        created_at: Date.now(),
      };
      annotations = [...annotations, annotation];
      snapshot();
    },

    remove(id: string) {
      annotations = annotations.filter((a) => a.id !== id);
      snapshot();
    },

    update(id: string, changes: Partial<Annotation>) {
      annotations = annotations.map((a) =>
        a.id === id ? { ...a, ...changes } : a
      );
      snapshot();
    },

    undo() {
      if (undoIndex > 0) {
        undoIndex--;
        annotations = structuredClone(undoStack[undoIndex]);
      }
    },

    redo() {
      if (undoIndex < undoStack.length - 1) {
        undoIndex++;
        annotations = structuredClone(undoStack[undoIndex]);
      }
    },

    clear() {
      annotations = [];
      nextNumber = 1;
      undoStack = [[]];
      undoIndex = 0;
    },

    loadAnnotations(data: Annotation[]) {
      annotations = data;
      nextNumber =
        data.length > 0 ? Math.max(...data.map((a) => a.number)) + 1 : 1;
      undoStack = [structuredClone(data)];
      undoIndex = 0;
    },
  };
}
