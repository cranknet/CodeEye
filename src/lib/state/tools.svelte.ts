export type ToolType =
  | "select"
  | "circle"
  | "rectangle"
  | "arrow"
  | "freehand"
  | "text";

export type QuickLabel = {
  name: string;
  severity: "critical" | "minor" | "suggestion";
};

const DEFAULT_LABELS: QuickLabel[] = [
  { name: "spacing", severity: "minor" },
  { name: "alignment", severity: "minor" },
  { name: "color", severity: "minor" },
  { name: "font", severity: "minor" },
  { name: "overflow", severity: "critical" },
  { name: "responsive", severity: "critical" },
  { name: "z-index", severity: "minor" },
  { name: "missing element", severity: "critical" },
];

export function createToolStore() {
  let activeTool = $state<ToolType>("select");
  let activeColor = $state("#f97316");
  let activeSeverity = $state<"critical" | "minor" | "suggestion">("minor");
  let quickLabels = $state<QuickLabel[]>(DEFAULT_LABELS);
  let activeQuickLabel = $state<string | null>(null);

  return {
    get activeTool() {
      return activeTool;
    },
    get activeColor() {
      return activeColor;
    },
    get activeSeverity() {
      return activeSeverity;
    },
    get quickLabels() {
      return quickLabels;
    },
    get activeQuickLabel() {
      return activeQuickLabel;
    },

    setTool(tool: ToolType) {
      activeTool = tool;
    },
    setColor(color: string) {
      activeColor = color;
    },
    setSeverity(s: "critical" | "minor" | "suggestion") {
      activeSeverity = s;
    },

    selectQuickLabel(name: string | null) {
      activeQuickLabel = name;
      if (name) {
        const label = quickLabels.find((l) => l.name === name);
        if (label) activeSeverity = label.severity;
      }
    },

    addQuickLabel(label: QuickLabel) {
      quickLabels = [...quickLabels, label];
    },

    removeQuickLabel(name: string) {
      quickLabels = quickLabels.filter((l) => l.name !== name);
    },
  };
}
