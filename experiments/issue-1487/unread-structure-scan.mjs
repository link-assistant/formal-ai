// Issue #1487: which coding prompts in the native and JS test suites leave
// structure no discovered coding part reads? Those are the requests the
// synthesis guard withholds a self-verified program from; the scan lists
// them so each can be checked against what its test expects.
import fs from 'node:fs';
import path from 'node:path';
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';
installTextHost();
const { unreadStructure } = await import('../../js/agentic/crate/unread_structure.mjs');
const { meaningsWithRole, evidencedIn } = await import('../../js/agentic/crate/seed_meanings.mjs');
const { normalizePrompt } = await import('../../js/agentic/crate/engine.mjs');
const roots = process.argv.slice(2).length ? process.argv.slice(2) : ['rust/tests', 'rust/src'];
const files = [];
const walk = (dir) => {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (/.(rs|mjs|json|lino)$/.test(entry.name)) files.push(full);
  }
};
roots.forEach(walk);
const CODING = /\b(function|program|script|def)\b/i;
const seen = new Set();
for (const file of files) {
  const text = fs.readFileSync(file, 'utf8');
  for (const match of text.matchAll(/"((?:[^"\\\n]|\\.){12,400})"/g)) {
    const prompt = match[1].replace(/\\"/g, '"').replace(/\\n/g, '\n');
    if (!CODING.test(prompt) || seen.has(prompt)) continue;
    seen.add(prompt);
    const normalized = normalizePrompt(prompt);
    const ids = meaningsWithRole('coding_structure').filter((meaning) => evidencedIn(meaning, normalized)).map((meaning) => meaning.slug);
    if (ids.length === 0) continue;
    const unread = unreadStructure(prompt, ids);
    if (unread) console.log(`${file}\t${JSON.stringify(prompt).slice(0, 160)}\t[${ids.join(',')}]\t${unread}`);
  }
}
