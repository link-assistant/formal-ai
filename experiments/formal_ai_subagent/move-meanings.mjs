#!/usr/bin/env node
// Moves whole top-level meaning records from one seed file to another, by rule
// (PR #1188, LEXEMES), so a seed file that grows past the 1500-line limit is
// split by meaning category instead of by numbered part. Each named record
// moves with every line it owns (its indented subtree and the comment lines
// directly above it); the target file is created with the `meanings` root when
// it does not exist. Both files are copied to the mirrors that held the source
// file (a new target goes wherever the source went).
//
// Usage: node experiments/formal_ai_subagent/move-meanings.mjs <from.lino> <to.lino> <meaning>...
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { basename } from 'node:path';

const MIRRORS = ['rust/embedded/data/seed', 'js/seed', 'packages/formal-ai-engine/assets/seed'];
const RECORD_INDENT = '  ';

function isRecordStart(line) {
  return line.startsWith(RECORD_INDENT) && !line.startsWith(`${RECORD_INDENT} `) && !line.trim().startsWith('#');
}

/** The [start, end) line range of the record `name`, comment lines above it included. */
function recordRange(lines, name) {
  const at = lines.findIndex((line) => isRecordStart(line) && line.trim() === name);
  if (at < 0) return null;
  let start = at;
  while (start > 0 && lines[start - 1].trim().startsWith('#') && lines[start - 1].startsWith(RECORD_INDENT)) start -= 1;
  let end = at + 1;
  while (end < lines.length && (lines[end].startsWith(`${RECORD_INDENT} `) || lines[end].trim() === '')) end += 1;
  while (end > at + 1 && lines[end - 1].trim() === '') end -= 1;
  return [start, end];
}

function main() {
  const [from, to, ...names] = process.argv.slice(2);
  if (!from || !to || names.length === 0) {
    console.error('usage: move-meanings.mjs <from.lino> <to.lino> <meaning>...');
    process.exit(2);
  }
  const previous = readFileSync(from, 'utf8');
  const lines = previous.split('\n');
  const moved = [];
  for (const name of names) {
    const range = recordRange(lines, name);
    if (range === null) {
      console.error(`${from}: no top-level record ${name}`);
      process.exit(1);
    }
    moved.push(...lines.slice(range[0], range[1]));
    lines.splice(range[0], range[1] - range[0]);
  }
  const target = existsSync(to) ? readFileSync(to, 'utf8').replace(/\n*$/u, '\n') : 'meanings\n';
  writeFileSync(from, lines.join('\n'));
  writeFileSync(to, `${target}${moved.join('\n')}\n`);
  for (const directory of MIRRORS) {
    const mirrored = `${directory}/${basename(from)}`;
    if (!existsSync(mirrored) || readFileSync(mirrored, 'utf8') !== previous) continue;
    copyFileSync(from, mirrored);
    copyFileSync(to, `${directory}/${basename(to)}`);
  }
  console.log(`${from} -> ${to}: moved ${names.length} record(s), ${moved.length} line(s)`);
}

main();
