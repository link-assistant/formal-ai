// Issue #1571: list every difference between the committed governed witness and
// one regenerated from the current tree, so a refresh is reviewed, not blind.
// Usage: node experiments/issue-1571/governed-witness-delta.mjs
import {readFileSync, readdirSync} from 'node:fs';
import {deserialize} from 'node:v8';
import {createHash} from 'node:crypto';
import {resolve, relative, isAbsolute} from 'node:path';
import {fileURLToPath} from 'node:url';
import {specificationCases} from '../../scripts/lib/rust-specification-cases.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const hash = value => createHash('sha256').update(value).digest('hex');
const baseline = deserialize(readFileSync(resolve(root, 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8')));
const inventory = specificationCases(root);
const cases = structuredClone(inventory.cases);
for (const item of cases) for (const fixture of item.program?.fixtures ?? []) if (isAbsolute(fixture.file)) fixture.file = 'repo:' + relative(root, fixture.file);
const sourceManifest = readdirSync(resolve(root, 'rust/tests/unit/specification'), {recursive: true}).map(String).filter(n => n.endsWith('.rs')).sort()
  .map(n => ({path: 'rust/tests/unit/specification/' + n, sha256: hash(readFileSync(resolve(root, 'rust/tests/unit/specification/' + n)))}));
const fresh = {inventory: {...inventory, cases}, sourceManifest};
const deltas = [];
const walk = (a, b, path) => {
  if (Object.is(a, b)) return;
  if (typeof a !== 'object' || typeof b !== 'object' || a === null || b === null) { deltas.push(path); return; }
  for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) walk(a[key], b[key], path + '/' + key);
};
walk(baseline, fresh, '');
const fixtureFiles = new Set();
for (const path of deltas) {
  const m = /^\/inventory\/cases\/(\d+)\/program\/fixtures\/(\d+)\/(\w+)$/u.exec(path);
  if (m) fixtureFiles.add(fresh.inventory.cases[+m[1]].program.fixtures[+m[2]].file + ' ' + m[3]);
  else console.log('OTHER', path);
}
console.log(JSON.stringify({deltaCount: deltas.length, fixtureFields: [...fixtureFiles].sort()}, null, 1));
