// Which prompts already used by tests and fixtures would the CNL dialogue capture?
// Usage: node experiments/issue-1584/captured-prompts.mjs [dir ...] > log
import { readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import '../../rust/tests/web/support/cnl-matrix.mjs';
import { cnlPolicy, controlledDialogue } from '../../js/agentic/crate/cnl_output.mjs';
const policy = cnlPolicy();
const roots = process.argv.slice(2).length ? process.argv.slice(2) : ['rust/tests', 'data/benchmarks', 'data/parity', 'js', 'tests'];
const seen = new Map();
const walk = (dir) => {
  let entries; try { entries = readdirSync(dir); } catch { return; }
  for (const name of entries) {
    const file = path.join(dir, name);
    if (name === 'node_modules' || name.startsWith('.')) continue;
    const stat = statSync(file);
    if (stat.isDirectory()) walk(file);
    else if (/\.(mjs|js|json|rs|lino|jsonl)$/u.test(name) && stat.size < 4e6) {
      const text = readFileSync(file, 'utf8');
      for (const match of text.matchAll(/"((?:[^"\\\n]|\\.){3,200})"|'((?:[^'\\\n]|\\.){3,200})'/gu)) {
        const prompt = (match[1] ?? match[2]).replace(/\\(.)/gu, '$1');
        if (!/\p{L}/u.test(prompt) || seen.has(prompt)) continue;
        let result = null;
        try { result = controlledDialogue(prompt, [], {}, policy); } catch (error) { result = {intent: 'error:' + error.message}; }
        seen.set(prompt, true);
        if (result) console.log(JSON.stringify({file, prompt, intent: result.intent, out: result.prompt ?? result.content}));
      }
    }
  }
};
for (const root of roots) walk(root);
