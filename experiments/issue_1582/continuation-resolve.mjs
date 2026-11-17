// Supplied merge procedure. Preserve main's policy explanations, then measure
// the combined census and review the complete inventory before witness refresh.
import {execFileSync} from 'node:child_process';
import {readFileSync, writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import {parseLedger, sourceFiles, audit} from '../../scripts/check-minimal-core-boundary.mjs';
for (const file of ['.cargo/audit.toml', 'data/meta/worker-line-budget/formal_ai_worker_obligations.lino', 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8']) {
  execFileSync('git', ['checkout', '--theirs', '--', file]);
}
const path = 'data/meta/core-boundary-ledger.lino';
let text = readFileSync(path, 'utf8').replace(/<<<<<<< HEAD\n[\s\S]*?=======\n([\s\S]*?)>>>>>>> origin\/main\n/, '$1');
const ledger = parseLedger(text), files = sourceFiles(process.cwd());
const active = ledger.entries.filter(e => ['promote', 'migrate'].includes(e.disposition));
const outside = active.filter(e => e.disposition === 'migrate');
const sum = entries => entries.reduce((n, e) => n + files.get(e.path), 0);
for (const [key, value] of Object.entries({source_file_count_max: active.length, source_lines_max: sum(active), outside_core_file_count_max: outside.length, outside_core_lines_max: sum(outside)})) {
  text = text.replace(new RegExp(`(  ${key} )\\d+`), '$1' + value);
}
const changes = active.filter(e => e.baselineLines !== files.get(e.path));
assert.deepEqual(changes, [], 'Auto-merged entry baselines must already match the measured tree');
text = text.replace('  boundary ', '  note "Issue #1582 merge with #1588 retains both changes and remeasures the combined handler census; no disposition or semantic-debt category changes."\n  boundary ');
assert.deepEqual(audit(parseLedger(text), files), []);
writeFileSync(path, text);
execFileSync(process.execPath, ['experiments/issue_1582/refresh-governed-witness.mjs'], {stdio: 'inherit'});
execFileSync('git', ['add', '.cargo/audit.toml', path, 'data/meta/worker-line-budget/formal_ai_worker_obligations.lino', 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8']);
console.log('Resolved four conflicts; combined census', active.length, sum(active), outside.length, sum(outside));
