#!/usr/bin/env node

/**
 * UI Feedback MCP Server
 *
 * Dual-protocol bridge:
 *   HTTP  :3847  ←  receives feedback from the playground (browser)
 *   stdio MCP    ←  exposes tools to Claude Code
 *
 * Storage: tools/.feedback/<id>/  (prompt.md + annotated.png + meta.json)
 */

import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:http";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FEEDBACK_DIR = resolve(__dirname, "../.feedback");
const PORT = Number.parseInt(process.env.UI_FEEDBACK_PORT || "3847", 10);
const MAX_BODY = 50 * 1024 * 1024; // 50 MB

mkdirSync(FEEDBACK_DIR, { recursive: true });

/* ================================================================
   Helpers
   ================================================================ */

function getFeedbackEntries() {
  if (!existsSync(FEEDBACK_DIR)) {
    return [];
  }
  return readdirSync(FEEDBACK_DIR)
    .filter((d) => d.startsWith("fb-"))
    .sort()
    .reverse()
    .map((d) => {
      const p = join(FEEDBACK_DIR, d, "meta.json");
      if (!existsSync(p)) {
        return null;
      }
      try {
        return JSON.parse(readFileSync(p, "utf-8"));
      } catch {
        return null;
      }
    })
    .filter(Boolean);
}

function getFeedback(id) {
  const dir = join(FEEDBACK_DIR, id);
  if (!existsSync(dir)) {
    return null;
  }

  const meta = JSON.parse(readFileSync(join(dir, "meta.json"), "utf-8"));
  const prompt = readFileSync(join(dir, "prompt.md"), "utf-8");
  const imgPath = join(dir, "annotated.png");
  const imageBase64 = existsSync(imgPath)
    ? readFileSync(imgPath).toString("base64")
    : null;

  return { meta, prompt, imageBase64, imagePath: imgPath };
}

/* ================================================================
   HTTP Server — receives feedback from the playground
   ================================================================ */

const httpServer = createServer(async (req, res) => {
  // CORS (playground runs from file:// or localhost)
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "POST, GET, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.writeHead(204);
    res.end();
    return;
  }

  // Health check
  if (req.method === "GET" && req.url === "/health") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        status: "ok",
        feedbackCount: getFeedbackEntries().length,
      })
    );
    return;
  }

  // Receive feedback
  if (req.method === "POST" && req.url === "/feedback") {
    const chunks = [];
    let size = 0;

    for await (const chunk of req) {
      size += chunk.length;
      if (size > MAX_BODY) {
        res.writeHead(413, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: "Payload too large (50 MB max)" }));
        return;
      }
      chunks.push(chunk);
    }

    try {
      const data = JSON.parse(Buffer.concat(chunks).toString());
      const id = `fb-${Date.now()}`;
      const dir = join(FEEDBACK_DIR, id);
      mkdirSync(dir, { recursive: true });

      // Save annotated image
      if (data.annotatedImage) {
        const b64 = data.annotatedImage.replace(/^data:image\/\w+;base64,/, "");
        writeFileSync(join(dir, "annotated.png"), Buffer.from(b64, "base64"));
      }

      // Save structured prompt
      writeFileSync(join(dir, "prompt.md"), data.prompt || "");

      // Save metadata
      writeFileSync(
        join(dir, "meta.json"),
        JSON.stringify(
          {
            id,
            pageName: data.pageName || "Untitled",
            fileName: data.fileName || "screenshot.png",
            imageWidth: data.imageWidth || 0,
            imageHeight: data.imageHeight || 0,
            annotationCount: data.annotations?.length || 0,
            timestamp: new Date().toISOString(),
          },
          null,
          2
        )
      );

      console.error(
        `[ui-feedback] Received: ${id} — ${data.annotations?.length || 0} annotations for "${data.pageName || "Untitled"}"`
      );
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ success: true, id }));
    } catch (err) {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: err.message }));
    }
    return;
  }

  res.writeHead(404);
  res.end("Not Found");
});

httpServer.listen(PORT, "127.0.0.1", () => {
  console.error(`[ui-feedback] HTTP listening on http://127.0.0.1:${PORT}`);
});

httpServer.on("error", (err) => {
  console.error(`[ui-feedback] HTTP error: ${err.message}`);
});

/* ================================================================
   MCP Server — exposes tools to Claude Code
   ================================================================ */

const mcp = new McpServer({
  name: "ui-feedback",
  version: "1.0.0",
});

// ── get_ui_feedback ──
mcp.tool(
  "get_ui_feedback",
  'Get UI feedback with annotated screenshot and structured issues. Returns both the markdown report and the annotated image. Call when the user says "check UI feedback" or "review UI issues".',
  { id: z.string().optional().describe("Feedback ID — omit for latest") },
  async ({ id }) => {
    let targetId = id;

    if (!targetId) {
      const entries = getFeedbackEntries();
      if (entries.length === 0) {
        return {
          content: [
            {
              type: "text",
              text: "No UI feedback available. Open the playground and send feedback first.",
            },
          ],
        };
      }
      targetId = entries[0].id;
    }

    const feedback = getFeedback(targetId);
    if (!feedback) {
      return {
        content: [{ type: "text", text: `Feedback "${targetId}" not found.` }],
        isError: true,
      };
    }

    const content = [{ type: "text", text: feedback.prompt }];

    if (feedback.imageBase64) {
      content.push({
        type: "image",
        data: feedback.imageBase64,
        mimeType: "image/png",
      });
    }

    return { content };
  }
);

// ── list_ui_feedback ──
mcp.tool(
  "list_ui_feedback",
  "List all queued UI feedback entries with IDs, page names, annotation counts, and timestamps.",
  {},
  async () => {
    const entries = getFeedbackEntries();
    if (entries.length === 0) {
      return { content: [{ type: "text", text: "No UI feedback queued." }] };
    }

    const lines = entries
      .map(
        (e, i) =>
          `${i + 1}. **${e.pageName}** — ${e.annotationCount} issue${e.annotationCount !== 1 ? "s" : ""} — \`${e.id}\`\n   _${e.timestamp}_`
      )
      .join("\n");

    return {
      content: [
        {
          type: "text",
          text: `# UI Feedback Queue\n\n${lines}\n\n_Use \`get_ui_feedback\` to view details and annotated screenshot._`,
        },
      ],
    };
  }
);

// ── resolve_ui_feedback ──
mcp.tool(
  "resolve_ui_feedback",
  "Mark UI feedback as resolved and remove it from the queue. Use after fixing the reported issues.",
  {
    id: z
      .string()
      .describe('Feedback ID to resolve, or "all" to clear entire queue'),
  },
  async ({ id }) => {
    if (id === "all") {
      const entries = getFeedbackEntries();
      for (const entry of entries) {
        rmSync(join(FEEDBACK_DIR, entry.id), { recursive: true, force: true });
      }
      return {
        content: [
          {
            type: "text",
            text: `Cleared ${entries.length} feedback entr${entries.length !== 1 ? "ies" : "y"}.`,
          },
        ],
      };
    }

    const dir = join(FEEDBACK_DIR, id);
    if (!existsSync(dir)) {
      return {
        content: [{ type: "text", text: `Feedback "${id}" not found.` }],
        isError: true,
      };
    }

    rmSync(dir, { recursive: true, force: true });
    return {
      content: [{ type: "text", text: `Feedback "${id}" resolved.` }],
    };
  }
);

// ── Connect ──
const transport = new StdioServerTransport();
await mcp.connect(transport);
