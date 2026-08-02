#!/usr/bin/env node

import { readFileSync } from "node:fs";

const [mode, path] = process.argv.slice(2);
if (!mode || !path) {
  throw new Error("usage: verify-proxy.mjs <code|report> <proxy.jsonl>");
}

const rows = readFileSync(path, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line));

if (mode === "code") {
  const calls = rows.flatMap((row) => row.response_tool_calls ?? []);
  if (!calls.some((call) => call.name === "apply_patch")) {
    throw new Error("Codex transcript has no apply_patch response tool call");
  }
  if (calls.filter((call) => call.name === "exec_command").length < 2) {
    throw new Error("Codex transcript has fewer than two exec_command calls");
  }
} else if (mode === "report") {
  const reportRows = rows.filter((row) => {
    try {
      return JSON.stringify(JSON.parse(row.request_body).input).includes("Report issue");
    } catch {
      return false;
    }
  });
  const calls = reportRows.flatMap((row) => row.response_tool_calls ?? []);
  if (!calls.some((call) => call.name === "request_user_input")) {
    throw new Error("Report issue transcript has no request_user_input call");
  }
  if (calls.some((call) => call.name === "web_search")) {
    throw new Error("Report issue transcript incorrectly invoked web_search");
  }
} else {
  throw new Error(`unknown verification mode: ${mode}`);
}
