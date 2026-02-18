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

/** Default prompt template matching the built-in Rust constant.
 * SYNC: Keep in sync with `src-tauri/src/storage.rs:DEFAULT_PROMPT_TEMPLATE`. */
export const DEFAULT_PROMPT_TEMPLATE = `# UI Feedback Report

**Page:** {{page_name}}
**Project:** {{project}}
**Branch:** {{branch}}
**File:** {{suggested_file}}
**Viewport:** {{viewport}}

## Notes

{{notes}}

## Issues

{{annotations}}

## Recent Changes

\`\`\`diff
{{recent_diff}}
\`\`\``;

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

/** Format all annotations into a markdown string. */
function formatAnnotations(
  annotations: Annotation[],
  viewport?: { width: number; height: number }
): string {
  if (annotations.length === 0) {
    return "";
  }
  return annotations
    .flatMap((ann) => formatAnnotation(ann, viewport))
    .join("\n")
    .trimEnd();
}

/**
 * Render a prompt template with variable substitution.
 *
 * Variables use `{{name}}` syntax. Supported:
 * - `{{page_name}}`, `{{project}}`, `{{branch}}`, `{{suggested_file}}`
 * - `{{viewport}}`, `{{notes}}`, `{{annotations}}`, `{{recent_diff}}`
 *
 * Empty sections are cleaned up: if a `## Heading` is followed only by
 * whitespace and an empty variable expansion, the entire section (heading
 * through next heading or end) is removed.
 */
export function renderTemplate(template: string, input: PromptInput): string {
  const vars: Record<string, string> = {
    page_name: input.pageName || "",
    project: input.gitContext?.project || "",
    branch: input.gitContext?.branch || "",
    suggested_file: input.gitContext?.suggested_file || "",
    viewport: input.viewport
      ? `${input.viewport.width}×${input.viewport.height}`
      : "",
    notes: input.generalNotes?.trim() || "",
    annotations: formatAnnotations(input.annotations, input.viewport),
    recent_diff: input.gitContext?.recent_diff || "",
  };

  // Substitute all {{variable}} placeholders
  let result = template.replace(
    /\{\{(\w+)\}\}/g,
    (_match, name: string) => vars[name] ?? ""
  );

  // Prepend free-text instruction if present
  if (input.freeTextInstruction?.trim()) {
    result = `${input.freeTextInstruction.trim()}\n\n${result}`;
  }

  // Strip empty sections: a ## heading followed by only whitespace until
  // the next ## heading or end of string. Also handles ```diff\n\n```
  // blocks that became empty.
  result = result.replace(/## [^\n]+\n+```\w*\n\s*\n```\s*/g, "");
  result = result.replace(/## [^\n]+\n+(?=## |\s*$)/g, "");

  // Strip lines that are just "**Label:** " with no value
  result = result.replace(/\*\*\w[^*]*:\*\*\s*\n/g, "");

  // Collapse 3+ consecutive newlines into 2
  result = result.replace(/\n{3,}/g, "\n\n");

  return result.trimEnd();
}

/**
 * Generate a structured markdown prompt from annotations and context.
 * Uses the default template for backward compatibility.
 */
export function generatePrompt(input: PromptInput): string {
  return renderTemplate(DEFAULT_PROMPT_TEMPLATE, input);
}
