// Probe the browser-worker solver with prompts phrased in different input styles.
// Usage: node experiments/issue-1584/probe-solver.mjs "prompt" ["prompt" ...]
import { readFileSync } from "node:fs";
import path from "node:path";
import { createWorkerContext, evaluate, plain } from "../../rust/tests/web/support/browser-runtime.mjs";
const ROOT = path.resolve(import.meta.dirname, "../..");
const worker = createWorkerContext({ fetch: async (url) => {
  const target = String(url);
  if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
    const rel = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
    const file = rel.startsWith("seed/") ? path.join(ROOT, "data", rel) : path.join(ROOT, "js", rel);
    try { const text = readFileSync(file, "utf8"); return { ok: true, status: 200, text: async () => text }; }
    catch { return { ok: false, status: 404, text: async () => "" }; }
  }
  return { ok: false, status: 404, text: async () => "" };
} });
await evaluate(worker, "loadSeed()");
for (const prompt of process.argv.slice(2)) {
  const answer = plain(await worker.solve(prompt, [], {}, {}, [], {}));
  console.log(JSON.stringify({ prompt, intent: answer.intent, content: String(answer.content).slice(0, 160) }));
}
