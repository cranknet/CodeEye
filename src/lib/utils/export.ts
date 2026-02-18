import type { Annotation } from "$lib/state/annotations.svelte";

export interface GitContext {
  branch: string;
  project: string;
  recent_diff: string | null;
  suggested_file: string | null;
  working_directory: string;
}

export interface PromptInput {
  annotations: Annotation[];
  freeTextInstruction?: string;
  generalNotes: string;
  gitContext?: GitContext;
  pageName: string;
  viewport?: { width: number; height: number };
}

const SEVERITY_LABELS: Record<string, string> = {
  critical: "CRITICAL",
  minor: "MINOR",
  suggestion: "SUGGESTION",
};

/** Format a single annotation as markdown lines */
function formatAnnotation(
  ann: Annotation,
  viewport?: { width: number; height: number }
): string[] {
  const lines: string[] = [];
  const severity = SEVERITY_LABELS[ann.severity] ?? ann.severity;
  const label = ann.label ? ` — ${ann.label}` : "";
  lines.push(`### #${ann.number} [${severity}]${label}`);
  lines.push("");

  const { x, y, w, h } = ann.bounds;
  lines.push(`- **Position:** (${Math.round(x)}, ${Math.round(y)})`);
  lines.push(`- **Size:** ${Math.round(w)}×${Math.round(h)}`);
  lines.push(`- **Type:** ${ann.type}`);

  if (viewport && viewport.width > 0) {
    const pctX = ((x / viewport.width) * 100).toFixed(1);
    const pctY = ((y / viewport.height) * 100).toFixed(1);
    lines.push(`- **Relative:** ${pctX}% from left, ${pctY}% from top`);
  }

  if (ann.comment?.trim()) {
    lines.push("");
    lines.push(ann.comment.trim());
  }

  lines.push("");
  return lines;
}

/**
 * Generate a structured markdown prompt from annotations and context.
 * Output follows the PRD Section 4.6 format.
 */
export function generatePrompt(input: PromptInput): string {
  const lines: string[] = [];

  if (input.freeTextInstruction?.trim()) {
    lines.push(input.freeTextInstruction.trim(), "");
  }

  lines.push("# UI Feedback Report", "");

  if (input.pageName) {
    lines.push(`**Page:** ${input.pageName}`);
  }

  if (input.gitContext) {
    const git = input.gitContext;
    lines.push(`**Project:** ${git.project}`);
    lines.push(`**Branch:** ${git.branch}`);
    if (git.suggested_file) {
      lines.push(`**File:** ${git.suggested_file}`);
    }
  }

  if (input.viewport) {
    lines.push(
      `**Viewport:** ${input.viewport.width}×${input.viewport.height}`
    );
  }

  lines.push("");

  if (input.generalNotes?.trim()) {
    lines.push("## Notes", "", input.generalNotes.trim(), "");
  }

  if (input.annotations.length > 0) {
    lines.push("## Issues", "");
    for (const ann of input.annotations) {
      lines.push(...formatAnnotation(ann, input.viewport));
    }
  }

  if (input.gitContext?.recent_diff) {
    lines.push(
      "## Recent Changes",
      "",
      "```diff",
      input.gitContext.recent_diff,
      "```",
      ""
    );
  }

  return lines.join("\n").trimEnd();
}
