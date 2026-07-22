#!/usr/bin/env node

// Deterministic, read-only research fixture for the issue #538 Agent CLI E2E.
// The real client still executes search and fetch tool calls; only the hosted
// Exa provider and outbound GitHub fetch are replaced with repository fixtures.

import { readFileSync } from "node:fs";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const tomatoUrl = "https://raw.githubusercontent.com/link-assistant/formal-ai/issue-538-eca4a11c39c6/data/cache/wikidata/lexeme/L170542.json";
const potatoUrl = "https://raw.githubusercontent.com/link-assistant/formal-ai/issue-538-eca4a11c39c6/data/cache/wikidata/lexeme/L3784.json";
const sourcePath = (name) => fileURLToPath(
  new URL(`../../data/cache/wikidata/lexeme/${name}`, import.meta.url),
);
const sources = new Map([
  [tomatoUrl, readFileSync(sourcePath("L170542.json"), "utf8")],
  [potatoUrl, readFileSync(sourcePath("L3784.json"), "utf8")],
]);

function result(id, value) {
  process.stdout.write(`${JSON.stringify({ jsonrpc: "2.0", id, result: value })}\n`);
}

function error(id, code, message) {
  process.stdout.write(
    `${JSON.stringify({ jsonrpc: "2.0", id, error: { code, message } })}\n`,
  );
}

function textResult(text) {
  return { content: [{ type: "text", text }], isError: false };
}

function handle(message) {
  const { id, method, params = {} } = message;
  if (method === "initialize") {
    result(id, {
      protocolVersion: params.protocolVersion || "2025-06-18",
      capabilities: { tools: {} },
      serverInfo: { name: "issue-538-meaning-fixture", version: "1.0.0" },
    });
    return;
  }
  if (method === "notifications/initialized" || method === "notifications/cancelled") {
    return;
  }
  if (method === "ping") {
    result(id, {});
    return;
  }
  if (method === "tools/list") {
    result(id, {
      tools: [
        {
          name: "websearch",
          description: "Search the deterministic Wikidata meaning fixture",
          inputSchema: {
            type: "object",
            properties: { query: { type: "string" } },
            required: ["query"],
            additionalProperties: false,
          },
          annotations: {
            title: "Search meaning fixture",
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: true,
          },
        },
        {
          name: "webfetch",
          description: "Fetch one deterministic Wikidata lexeme document",
          inputSchema: {
            type: "object",
            properties: {
              url: { type: "string" },
              format: { type: "string", enum: ["text", "markdown", "html"] },
            },
            required: ["url"],
            additionalProperties: false,
          },
          annotations: {
            title: "Fetch meaning fixture",
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: true,
          },
        },
      ],
    });
    return;
  }
  if (method === "tools/call") {
    const name = params.name;
    const args = params.arguments || {};
    process.stderr.write(`[issue-538-mcp] ${name} ${JSON.stringify(args)}\n`);
    if (name === "websearch") {
      const url = String(args.query).toLowerCase().includes("potato")
        ? potatoUrl
        : tomatoUrl;
      result(id, textResult(`Wikidata lexeme data ${url}`));
      return;
    }
    if (name === "webfetch") {
      const body = sources.get(args.url);
      if (body) {
        result(id, textResult(body));
      } else {
        result(id, {
          content: [{ type: "text", text: `Error: fixture URL not found: ${args.url}` }],
          isError: true,
        });
      }
      return;
    }
    error(id, -32601, `Unknown tool: ${name}`);
    return;
  }
  if (id !== undefined) {
    error(id, -32601, `Unknown method: ${method}`);
  }
}

const input = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
input.on("line", (line) => {
  if (!line.trim()) return;
  try {
    handle(JSON.parse(line));
  } catch (exception) {
    process.stderr.write(`[issue-538-mcp] invalid request: ${exception.message}\n`);
  }
});
