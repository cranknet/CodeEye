import { describe, expect, it } from "vitest";
import type { Annotation } from "$lib/state/annotations.svelte";
import {
  DEFAULT_PROMPT_TEMPLATE,
  generatePrompt,
  type PromptInput,
  renderTemplate,
} from "./export";

function makeAnnotation(overrides: Partial<Annotation> = {}): Annotation {
  return {
    id: "test-1",
    number: 1,
    type: "rectangle",
    bounds: { x: 100, y: 200, w: 300, h: 150 },
    label: "spacing issue",
    comment: "Too much padding on the left",
    severity: "minor",
    color: "#eab308",
    ...overrides,
  };
}

describe("generatePrompt", () => {
  it("generates header with page name", () => {
    const input: PromptInput = {
      pageName: "/dashboard",
      generalNotes: "",
      annotations: [],
    };
    const result = generatePrompt(input);
    expect(result).toContain("# UI Feedback Report");
    expect(result).toContain("**Page:** /dashboard");
  });

  it("includes general notes section", () => {
    const input: PromptInput = {
      pageName: "",
      generalNotes: "Overall the layout is off-center",
      annotations: [],
    };
    const result = generatePrompt(input);
    expect(result).toContain("## Notes");
    expect(result).toContain("Overall the layout is off-center");
  });

  it("formats annotations with severity and coordinates", () => {
    const ann = makeAnnotation();
    const input: PromptInput = {
      pageName: "/login",
      generalNotes: "",
      annotations: [ann],
    };
    const result = generatePrompt(input);
    expect(result).toContain("### #1 [MINOR] — spacing issue");
    expect(result).toContain("**Position:** (100, 200)");
    expect(result).toContain("**Size:** 300×150");
    expect(result).toContain("**Type:** rectangle");
    expect(result).toContain("Too much padding on the left");
  });

  it("includes percentage coordinates when viewport provided", () => {
    const ann = makeAnnotation({
      bounds: { x: 500, y: 300, w: 100, h: 50 },
    });
    const input: PromptInput = {
      pageName: "",
      generalNotes: "",
      annotations: [ann],
      viewport: { width: 1000, height: 600 },
    };
    const result = generatePrompt(input);
    expect(result).toContain("50.0% from left");
    expect(result).toContain("50.0% from top");
  });

  it("includes git context", () => {
    const input: PromptInput = {
      pageName: "",
      generalNotes: "",
      annotations: [],
      gitContext: {
        project: "my-app",
        branch: "feat/new-ui",
        working_directory: "/home/user/my-app",
        suggested_file: "src/App.tsx",
        recent_diff: "+ added line\n- removed line",
      },
    };
    const result = generatePrompt(input);
    expect(result).toContain("**Project:** my-app");
    expect(result).toContain("**Branch:** feat/new-ui");
    expect(result).toContain("**File:** src/App.tsx");
    expect(result).toContain("## Recent Changes");
    expect(result).toContain("```diff");
    expect(result).toContain("+ added line");
  });

  it("includes free-text instruction at the top", () => {
    const input: PromptInput = {
      pageName: "/settings",
      generalNotes: "",
      annotations: [],
      freeTextInstruction: "Fix these issues in the settings page",
    };
    const result = generatePrompt(input);
    const lines = result.split("\n");
    expect(lines[0]).toBe("Fix these issues in the settings page");
  });

  it("handles multiple annotations ordered by number", () => {
    const anns: Annotation[] = [
      makeAnnotation({
        id: "a",
        number: 1,
        severity: "critical",
        label: "broken",
      }),
      makeAnnotation({
        id: "b",
        number: 2,
        severity: "suggestion",
        label: "nice to have",
      }),
    ];
    const input: PromptInput = {
      pageName: "",
      generalNotes: "",
      annotations: anns,
    };
    const result = generatePrompt(input);
    expect(result).toContain("#1 [CRITICAL]");
    expect(result).toContain("#2 [SUGGESTION]");
    const idx1 = result.indexOf("#1");
    const idx2 = result.indexOf("#2");
    expect(idx1).toBeLessThan(idx2);
  });

  it("handles empty input gracefully", () => {
    const input: PromptInput = {
      pageName: "",
      generalNotes: "",
      annotations: [],
    };
    const result = generatePrompt(input);
    expect(result).toContain("# UI Feedback Report");
    expect(result).not.toContain("## Notes");
    expect(result).not.toContain("## Issues");
  });
});

describe("renderTemplate", () => {
  it("substitutes all variables from a custom template", () => {
    const template = "Page: {{page_name}}\nBranch: {{branch}}";
    const input: PromptInput = {
      pageName: "Home",
      generalNotes: "",
      annotations: [],
      gitContext: {
        project: "proj",
        branch: "main",
        working_directory: "/tmp",
        suggested_file: null,
        recent_diff: null,
      },
    };
    const result = renderTemplate(template, input);
    expect(result).toBe("Page: Home\nBranch: main");
  });

  it("strips empty sections from default template", () => {
    const input: PromptInput = {
      pageName: "/page",
      generalNotes: "",
      annotations: [],
    };
    const result = renderTemplate(DEFAULT_PROMPT_TEMPLATE, input);
    expect(result).not.toContain("## Notes");
    expect(result).not.toContain("## Issues");
    expect(result).not.toContain("## Recent Changes");
  });

  it("strips lines with empty label values", () => {
    const input: PromptInput = {
      pageName: "Test",
      generalNotes: "",
      annotations: [],
    };
    const result = renderTemplate(DEFAULT_PROMPT_TEMPLATE, input);
    expect(result).not.toContain("**Project:**");
    expect(result).not.toContain("**Branch:**");
    expect(result).not.toContain("**File:**");
    expect(result).not.toContain("**Viewport:**");
    expect(result).toContain("**Page:** Test");
  });

  it("renders annotations in custom template", () => {
    const template = "Issues:\n{{annotations}}";
    const ann = makeAnnotation({ number: 1, severity: "critical" });
    const input: PromptInput = {
      pageName: "",
      generalNotes: "",
      annotations: [ann],
    };
    const result = renderTemplate(template, input);
    expect(result).toContain("### #1 [CRITICAL]");
    expect(result).toContain("**Position:**");
  });

  it("renders viewport as WxH", () => {
    const template = "Size: {{viewport}}";
    const input: PromptInput = {
      pageName: "",
      generalNotes: "",
      annotations: [],
      viewport: { width: 1920, height: 1080 },
    };
    const result = renderTemplate(template, input);
    expect(result).toBe("Size: 1920×1080");
  });

  it("handles unknown variables by removing them", () => {
    const template = "Hello {{unknown_var}} world";
    const input: PromptInput = {
      pageName: "",
      generalNotes: "",
      annotations: [],
    };
    const result = renderTemplate(template, input);
    expect(result).toBe("Hello  world");
  });

  it("prepends free-text instruction", () => {
    const template = "# Report\n{{page_name}}";
    const input: PromptInput = {
      pageName: "Home",
      generalNotes: "",
      annotations: [],
      freeTextInstruction: "Fix this urgently",
    };
    const result = renderTemplate(template, input);
    expect(result.startsWith("Fix this urgently")).toBe(true);
    expect(result).toContain("# Report");
  });
});
