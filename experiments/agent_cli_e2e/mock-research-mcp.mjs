#!/usr/bin/env node

// Deterministic, read-only web research fixture for issue #781 CLI E2E tests.
// All four external clients execute these tools themselves, giving CI stable
// evidence while preserving each client's real protocol and tool-call loop.
// Separate captured runs retain Agent/OpenCode's network-backed evidence.

import readline from "node:readline";

const sources = new Map([
  [
    "https://acer.example.test/a325-45/specifications",
    "Acer Aspire 3 A325-45 specifications: the supplied adapter is rated 24 W.",
  ],
  [
    "https://parts.example.test/acer-a325-45/connector",
    "Acer Aspire 3 A325-45 power input: 12 V DC, 2 A, center-positive 3.5 x 1.35 mm barrel connector.",
  ],
  [
    "https://shop.example.test/compatible-a325-45-adapter",
    "Candidate adapter listing: 12 V, 2 A, 24 W, center-positive 3.5 x 1.35 mm plug; compatible with Acer Aspire 3 A325-45.",
  ],
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
      serverInfo: { name: "issue-781-research-fixture", version: "1.0.0" },
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
          description: "Search deterministic web evidence for the issue #781 charger task",
          inputSchema: {
            type: "object",
            properties: { query: { type: "string" } },
            required: ["query"],
            additionalProperties: false,
          },
          annotations: {
            title: "Search issue 781 fixture",
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: true,
          },
        },
        {
          name: "webfetch",
          description: "Fetch one deterministic issue #781 evidence page",
          inputSchema: {
            type: "object",
            properties: { url: { type: "string" } },
            required: ["url"],
            additionalProperties: false,
          },
          annotations: {
            title: "Fetch issue 781 fixture page",
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
    process.stderr.write(`[issue-781-mcp] ${name} ${JSON.stringify(args)}\n`);
    if (name === "websearch") {
      result(
        id,
        textResult(
          [
            "Acer specifications https://acer.example.test/a325-45/specifications",
            "Connector reference https://parts.example.test/acer-a325-45/connector",
            "Candidate listing https://shop.example.test/compatible-a325-45-adapter",
          ].join("\n"),
        ),
      );
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
    process.stderr.write(`[issue-781-mcp] invalid request: ${exception.message}\n`);
  }
});
