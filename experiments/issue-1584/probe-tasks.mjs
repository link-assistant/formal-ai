// Every declared task paraphrase (numbers filled in) through the browser-worker solver.
// Usage: node experiments/issue-1584/probe-tasks.mjs [language ...]
import { readFileSync } from "node:fs";
import path from "node:path";
import { createWorkerContext, evaluate, plain } from "../../rust/tests/web/support/browser-runtime.mjs";
import { policy } from "../../rust/tests/web/support/cnl-matrix.mjs";
import { canonicalPrompt, utteranceGraphs } from "../../js/agentic/crate/cnl_grammar.mjs";
const ROOT = path.resolve(import.meta.dirname, "../..");
const worker = createWorkerContext({ fetch: async (url) => {
  const rel = new URL(String(url), "http://localhost/").pathname.replace(/^\/+/u, "");
  const file = rel.startsWith("seed/") ? path.join(ROOT, "data", rel) : path.join(ROOT, "js", rel);
  try { const text = readFileSync(file, "utf8"); return { ok: true, status: 200, text: async () => text }; }
  catch { return { ok: false, status: 404, text: async () => "" }; }
} });
await evaluate(worker, "loadSeed()");

const languages = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(policy.grammars);
let failures = 0;
for (const language of languages) {
  const grammar = policy.grammars[language];
  for (const template of grammar.templates.filter(item => item.styles.includes('task'))) {
    for (const [index, variant] of template.positive.entries()) {
      const prompt = variant.replace(/\{left\.number\}/gu, String(7 + index)).replace(/\{right\.number\}/gu, '3');
      const graphs = utteranceGraphs(prompt, grammar);
      const expected = graphs.length === 1 ? canonicalPrompt(graphs[0], policy.shared) : null;
      const answer = plain(await worker.solve(prompt, [], {}, {}, [], {}));
      const ok = expected !== null && answer.intent !== 'unknown';
      if (!ok) failures += 1;
      console.log(JSON.stringify({ok, language, frame: template.frame, prompt, intent: answer.intent,
        evidence: (answer.evidence || []).filter(item => String(item).startsWith('cnl:')), content: String(answer.content).slice(0, 80)}));
    }
  }
}
console.log(failures ? `FAIL ${failures}` : 'PASS');
