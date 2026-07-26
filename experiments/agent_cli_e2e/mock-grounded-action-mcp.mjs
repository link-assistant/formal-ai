#!/usr/bin/env node

// Deterministic, read-only evidence for issue #840's definition and
// comparison Agent CLI journeys. The external client executes these tools;
// Formal AI still owns every routing, decomposition, and synthesis decision.

import readline from "node:readline";

const sources = new Map([
  [
    "https://dictionary.example.test/ru/fuflomicin",
    "Фуфломицин — разговорное неодобрительное название лекарства или метода лечения, клиническая эффективность которого не доказана.",
  ],
  [
    "https://evidence.example.test/ru/unproven-medicine",
    "Термин «фуфломицин» применяют к препаратам без надёжных доказательств эффективности; это не официальное фармакологическое название.",
  ],
  [
    "https://language.example.test/ru/fuflomicin-usage",
    "В разговорной речи слово «фуфломицин» выражает сомнение в доказательной базе и не определяет конкретное действующее вещество.",
  ],
]);

function searchResults(query) {
  const normalized = String(query).toLowerCase();
  if (normalized.includes("фуфломицин")) {
    return [
      "Словарное определение https://dictionary.example.test/ru/fuflomicin",
      "Справка о доказательности https://evidence.example.test/ru/unproven-medicine",
      "Употребление термина https://language.example.test/ru/fuflomicin-usage",
    ];
  }
  if (normalized.includes("фбс")) {
    return [
      "ФБС evidence: продавец хранит товар на своём складе, собирает заказ и передаёт его маркетплейсу для доставки.",
    ];
  }
  if (normalized.includes("фбо")) {
    return [
      "ФБО evidence: продавец заранее поставляет товар на склад маркетплейса, который хранит, собирает и доставляет заказ.",
    ];
  }
  return ["No fixture evidence for this query."];
}

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
      serverInfo: { name: "issue-840-grounded-action-fixture", version: "1.0.0" },
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
          description: "Search deterministic issue 840 evidence",
          inputSchema: {
            type: "object",
            properties: { query: { type: "string" } },
            required: ["query"],
            additionalProperties: false,
          },
          annotations: {
            title: "Search grounded-action fixture",
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: true,
          },
        },
        {
          name: "webfetch",
          description: "Fetch one deterministic issue 840 evidence page",
          inputSchema: {
            type: "object",
            properties: { url: { type: "string" } },
            required: ["url"],
            additionalProperties: false,
          },
          annotations: {
            title: "Fetch grounded-action fixture page",
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
    process.stderr.write(`[issue-840-mcp] ${name} ${JSON.stringify(args)}\n`);
    if (name === "websearch") {
      result(id, textResult(searchResults(args.query).join("\n")));
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
    process.stderr.write(`[issue-840-mcp] invalid request: ${exception.message}\n`);
  }
});
