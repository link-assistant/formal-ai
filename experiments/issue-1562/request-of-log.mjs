// Issue #1562: extract the recorded chat request bodies of a hive-mind solve.log.
// Usage: node experiments/issue-1562/request-of-log.mjs <solve.log> [index] [out.json]
import { readFileSync, writeFileSync } from 'node:fs';

const lines = readFileSync(process.argv[2], 'utf8').split('\n');
const bodies = [];
for (const line of lines) {
  const match = /"bodyPreview": (".*"),?$/u.exec(line);
  if (!match) continue;
  let text;
  try { text = JSON.parse(match[1]); } catch { continue; }
  if (!text.startsWith('{"model":"formal-ai"')) continue;
  let body = null;
  try { body = JSON.parse(text); } catch { body = null; }
  bodies.push({ length: text.length, complete: body !== null, body });
}
console.log(bodies.map((entry, index) => `${index}: ${entry.length} ${entry.complete ? 'complete' : 'TRUNCATED'} ${entry.body?.messages?.length ?? '?'} messages`).join('\n'));
const index = Number(process.argv[3] ?? -1);
if (index >= 0 && process.argv[4]) writeFileSync(process.argv[4], JSON.stringify(bodies[index].body, null, 2));
