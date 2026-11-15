#!/usr/bin/env node
// Extract a deterministic offline replay fixture from a downloaded Hive draft
// session artifact (issue #1486).
//
// Usage:
//   gh run download <run> --name hive-mind-draft-session-<issue>-<attempt> --dir /tmp/session
//   node scripts/extract-draft-replay.mjs --session /tmp/session --issue 1486 --run <run> \
//     --out data/benchmarks/fixtures/draft-replays/issue-1486-read-policy.json
//
// The family is resolved from the captured reason through
// data/benchmarks/focused-test-map.json unless --family is given; an unmapped
// failure is an error, so every captured replay has focused tests.
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { extractDraftReplay } from './lib/draft-replay.mjs';

function argument(name) {
  const index = process.argv.indexOf(name);
  return index < 0 ? null : process.argv[index + 1] ?? null;
}

const session = argument('--session');
const issue = argument('--issue');
const out = argument('--out');
if (!session || !issue || !out) {
  console.error('usage: extract-draft-replay.mjs --session DIR --issue N --out FILE [--run ID] [--source SHA] [--family ID]');
  process.exit(2);
}
const dialogs = join(session, 'formal-ai-dialog');
// The conversation dialog with the most requests ended the attempt.
const candidates = readdirSync(dialogs).filter(name => /^dialog_[0-9a-f]+\.lino$/u.test(name))
  .map(name => join(dialogs, name)).sort((a, b) => statSync(b).size - statSync(a).size);
if (!candidates.length) throw Error('SessionHasNoDialog');
let sourceCommit = argument('--source');
if (!sourceCommit) {
  try {
    // `main_commit=` is the Formal AI source; `hive_mind_main_commit=` is Hive's.
    sourceCommit = /^main_commit=([0-9a-f]{40})$/mu.exec(readFileSync(join(session, 'source-identity.txt'), 'utf8'))?.[1] ?? null;
  } catch {
    sourceCommit = null;
  }
}
const mapping = JSON.parse(readFileSync('data/benchmarks/focused-test-map.json', 'utf8'));
const fixture = extractDraftReplay(readFileSync(candidates[0], 'utf8'), {
  issue: Number(issue),
  run: argument('--run') ? Number(argument('--run')) : null,
  sourceCommit,
  family: argument('--family'),
  mapping
});
writeFileSync(out, JSON.stringify(fixture) + '\n');
console.log(JSON.stringify({ out, family: fixture.family, reason: fixture.captured.reason, messages: fixture.messages.length }));
