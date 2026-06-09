// Regenerate the worker's embedded `MEANINGS_LINO` fallback block so it mirrors
// the de-obfuscated `data/seed/meanings*.lino` files verbatim (issue #398).
//
// The worker keeps an inline copy of the meaning lexicon so the browser demo
// stays functional when the `.lino` fetch fails (e.g. `file://`). After the
// codepoint byte-dumps were migrated to readable scalars, that inline copy must
// be refreshed to match — otherwise the fallback ships stale byte-dumps.
//
// This is the JS twin of `refresh_worker_meanings` in
// `scripts/migrate-meaning-seed.rs`: concatenate the meaning seed files, escape
// each line as a JS string literal, and splice the block back into the worker.
//
// Run: `node experiments/regen_worker_meanings.mjs`

import fs from 'node:fs';

const MEANING_SEED_FILES = [
  'meanings',
  'meanings-units',
  'meanings-calendar',
  'meanings-calculator',
  'meanings-facts',
  'meanings-software-project',
  'meanings-program-synthesis',
  'meanings-intent',
  'meanings-how',
  'meanings-meta',
  'meanings-web-navigation',
  'meanings-web-search',
  'meanings-web-search-query',
  'meanings-web-research',
  'meanings-web-followup',
  'meanings-translation',
  'meanings-ontology',
  'meanings-semantic-meta',
  'meanings-lexical-meta',
  'meanings-links-root',
  'meanings-wikidata',
  'meanings-behavior-rules',
  'meanings-proof',
  'meanings-policy',
  'meanings-docs',
  'meanings-skill-compiler',
  'meanings-finance',
  'meanings-definition-merge',
  'meanings-tool-access',
  'meanings-feature-capability',
  'meanings-playwright',
  'meanings-research-table',
  'meanings-conversation',
  'meanings-summary',
  'meanings-coding-catalog',
].map((name) => `data/seed/${name}.lino`);

const WORKER_PATH = 'src/web/formal_ai_worker.js';

// Mirror of `js_string` in scripts/migrate-meaning-seed.rs.
function jsString(value) {
  let out = '"';
  for (const ch of value) {
    const code = ch.charCodeAt(0);
    if (ch === '"') out += '\\"';
    else if (ch === '\\') out += '\\\\';
    else if (ch === '\n') out += '\\n';
    else if (ch === '\r') out += '\\r';
    else if (ch === '\t') out += '\\t';
    else if (code < 0x20) out += '\\u' + code.toString(16).padStart(4, '0');
    else out += ch;
  }
  return out + '"';
}

const lines = [];
for (const file of MEANING_SEED_FILES) {
  let content = fs.readFileSync(file, 'utf8');
  if (content.endsWith('\n')) content = content.slice(0, -1);
  for (const line of content.split('\n')) lines.push(line);
}

let block = 'const MEANINGS_LINO = [\n';
for (const line of lines) block += '  ' + jsString(line) + ',\n';
block += '].join("\\n");';

const worker = fs.readFileSync(WORKER_PATH, 'utf8');
const startMarker = 'const MEANINGS_LINO = [';
const endMarker = '].join("\\n");';
const start = worker.indexOf(startMarker);
if (start < 0) throw new Error('MEANINGS_LINO start not found');
const end = worker.indexOf(endMarker, start);
if (end < 0) throw new Error('MEANINGS_LINO end not found');
const next = worker.slice(0, start) + block + worker.slice(end + endMarker.length);

if (next !== worker) {
  fs.writeFileSync(WORKER_PATH, next);
  console.log(`refreshed MEANINGS_LINO (${lines.length} lines) in ${WORKER_PATH}`);
} else {
  console.log('MEANINGS_LINO already up to date');
}
