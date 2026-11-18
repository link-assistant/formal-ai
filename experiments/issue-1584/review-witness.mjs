// Compare complete governed source records before an explicit maintained refresh.
import fs from 'node:fs';
import {deserialize, serialize} from 'node:v8';
import {resolve, relative, isAbsolute} from 'node:path';
import {createHash} from 'node:crypto';
import {specificationCases} from '../../scripts/lib/rust-specification-cases.mjs';
import {scalarDifferenceReport} from '../../scripts/lib/regenerate-governed-specification.mjs';
const root = resolve(import.meta.dirname, '../..');
const baseline = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const prefix = process.argv[2];
if (!prefix) throw Error('Evidence prefix required');
const previousBytes = fs.readFileSync(resolve(root, baseline));
const previous = deserialize(previousBytes);
const inventory = specificationCases(root);
for (const item of inventory.cases) for (const fixture of item.program?.fixtures ?? []) {
  if (isAbsolute(fixture.file)) fixture.file = 'repo:' + relative(root, fixture.file);
}
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const sourceManifest = fs.readdirSync(resolve(root, 'rust/tests/unit/specification'), {recursive: true})
  .map(String).filter(name => name.endsWith('.rs')).sort().map(name => {
    const path = 'rust/tests/unit/specification/' + name;
    return {path, sha256: hash(fs.readFileSync(resolve(root, path)))};
  });
const current = {inventory, sourceManifest};
const report = {previousSha256: hash(previousBytes),
  before: {total: previous.inventory.total, cases: previous.inventory.cases.length, unsupported: previous.inventory.unsupported.length},
  after: {total: inventory.total, cases: inventory.cases.length, unsupported: inventory.unsupported.length},
  ...scalarDifferenceReport(previous, current)};
fs.writeFileSync(prefix + '.before.v8', previousBytes);
fs.writeFileSync(prefix + '.after.v8', serialize(current));
fs.writeFileSync(prefix + '.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({...report, details: report.details.length}));
