#!/usr/bin/env node

import { readFileSync } from "node:fs";

const [mode, path, prompt] = process.argv.slice(2);
if (!mode || !path || (mode === "report" && !prompt)) {
  throw new Error("usage: verify-proxy.mjs <code|report> <proxy.jsonl> [report prompt]");
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
      const request = JSON.parse(row.request_body);
      return request.input.some(item => item.role === "user"
        && item.content.some(part => part.text === prompt));
    } catch {
      return false;
    }
  });
  if (!reportRows.length) throw new Error("missing Codex report request");
  if (!reportRows.some(row => JSON.parse(row.request_body).tools.some(tool =>
    tool.name === "request_user_input" && tool.description.endsWith("This tool is only available in Plan mode.")))) {
    throw new Error("Codex did not advertise its Plan-only input tool");
  }
  const calls = reportRows.flatMap((row) => row.response_tool_calls ?? []);
  if (calls.length) {
    throw new Error(`Report invoked tools before confirmation: ${calls.map(call => call.name).join(", ")}`);
  }
} else {
  throw new Error(`unknown verification mode: ${mode}`);
}
