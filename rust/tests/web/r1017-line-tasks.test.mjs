// Line tasks put to Formal AI itself while building it (R1017).
//
// "Count the lines in this text: '...'" was answered by the shell composer
// with `ls .`: "count" is one of its action cues and "lines" one of its
// file-context words, yet the request names no file at all. Two general
// mechanisms fix it, never a rule for that one request: the English
// count_lines row of data/seed/operation-vocabulary.lino gained the token
// combo the other languages use, and the shell composer steps aside whenever
// the text-manipulation handler reads a seeded text operation over an inline
// payload (Rust `names_text_operation`).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");

const worker = createWorkerContext({
  fetch: async (url) => {
    const target = String(url);
    if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
      const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return { ok: true, status: 200, text: async () => text };
      } catch {
        return { ok: false, status: 404, text: async () => "" };
      }
    }
    // Offline: every external provider is unreachable.
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}

for (const prompt of [
  "Count the lines in this text: 'a\nb\nc'",
  "Count lines: 'a\nb\nc'",
  "Line count of this text: 'a\nb\nc'",
]) {
  test(`R1017: an inline line count is a text operation (${JSON.stringify(prompt)})`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "text_manipulation", JSON.stringify(answer));
    assert.equal(String(answer.content).trim(), "3", JSON.stringify(answer));
  });
}

test("R1017: the shell composer steps aside for an inline text payload", async () => {
  await ready;
  const prompt = "Count the lines in this text: 'a\nb\nc'";
  const composed = plain(await evaluate(worker, `handleShellCommandCompose(${JSON.stringify(prompt)}, ${JSON.stringify(prompt.toLowerCase())})`));
  assert.equal(composed, null, JSON.stringify(composed));
});

test("R1017: a file operand still composes a shell command", async () => {
  const answer = await solve("find .log files larger than 10 MB under /var");
  assert.equal(answer.intent, "shell_command_compose", JSON.stringify(answer));
  assert.match(String(answer.content), /find \/var/u, JSON.stringify(answer));
});
