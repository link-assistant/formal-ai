// Lists program_task_* seed aliases that occur (as substrings) in a prompt.
import fs from 'node:fs';
const text = fs.readFileSync('data/seed/meanings-coding-catalog.lino', 'utf8').split('\n');
let slug = null; const aliases = [];
for (const line of text) {
  const head = line.match(/^  (\S+)/);
  if (head) slug = head[1].startsWith('program_task_') ? head[1] : null;
  if (!slug) continue;
  const m = line.match(/^\s+(?:lexeme \S+|words)\s*(.*)$/);
  if (!m) continue;
  for (const q of m[1].matchAll(/"([^"]+)"|(\S+)/g)) aliases.push([slug, (q[1] ?? q[2]).toLowerCase()]);
}
const prompt = process.argv[2].toLowerCase();
console.log(aliases.filter(([, a]) => prompt.includes(a)));
