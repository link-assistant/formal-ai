// Issue #1487: review the governed specification witness delta after a
// refresh. node experiments/issue-1487/governed-witness-delta.mjs OLD.v8 NEW.v8
import { readFileSync } from 'node:fs';
import { deserialize } from 'node:v8';
import { scalarDifferenceReport } from '../../scripts/lib/regenerate-governed-specification.mjs';

const [before, after] = process.argv.slice(2).map((file) => deserialize(readFileSync(file)));
const report = scalarDifferenceReport(before, after);
console.log(JSON.stringify({ casesBefore: before.inventory.cases.length, casesAfter: after.inventory.cases.length,
  unsupportedBefore: before.inventory.unsupported.length, unsupportedAfter: after.inventory.unsupported.length,
  differences: report.differences, truncated: report.detailsTruncated }, null, 1));
for (const detail of report.details) console.log(detail.path, JSON.stringify(detail.before), '->', JSON.stringify(detail.after));
