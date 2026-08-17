// Print the guide the browser worker synthesises from the committed captures,
// so it can be compared with the Rust solver's output for the same task.
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { createWorkerContext, evaluate, plain } from "../tests/web/support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "..");
const CACHE = path.join(REPO_ROOT, "tests/fixtures/issue-991/source-cache");
const captures = new Map();
for (const name of readdirSync(CACHE).filter((n) => n.endsWith(".meta"))) {
  const meta = readFileSync(path.join(CACHE, name), "utf8");
  const field = (k) => (meta.split("\n").find((l) => l.startsWith(`${k}=`)) || "").slice(k.length + 1);
  captures.set(field("url"), readFileSync(path.join(CACHE, "objects", `${field("sha256")}.body`), "utf8"));
}
const context = createWorkerContext({
  fetch: (url) => {
    const target = String(url);
    if (!target.startsWith("http")) {
      const relative = target.split("?")[0].replace(/^\.?\//, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "src/web", relative);
      try { return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(readFileSync(onDisk, "utf8")) }); }
      catch { return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve("") }); }
    }
    const body = captures.get(target);
    return Promise.resolve(body ? { ok: true, status: 200, text: () => Promise.resolve(body) } : { ok: false, status: 404, text: () => Promise.resolve("") });
  },
});
await evaluate(context, "loadSeed()");
for (const task of process.argv.slice(2)) {
  context.__task = task;
  const guide = plain(await evaluate(context, "synthesizeHowToGuide(__task, {})"));
  console.log(`\n=== ${task}`);
  for (const outcome of guide.outcomes) console.log(`  outcome ${outcome.sourceId} ${outcome.status} pages=${outcome.pages} steps=${outcome.steps} ${outcome.detail}`);
  guide.steps.forEach((s, i) => console.log(`  ${i + 1}. [${s.sourceId} d${s.depth}] ${s.text}`));
}
