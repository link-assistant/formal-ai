// Print or rewrite the reviewed CNL infeasibility ledger (issue #1584).
// Usage: node experiments/issue-1584/feasibility.mjs [--write] [--detail <language> <profile> <density>]
// --write regenerates data/parity/issue-1584-cnl-infeasible.json; review the diff before committing.
import {writeFileSync} from 'node:fs';
import {compactJsonLeaves} from '../../scripts/lib/compact-json-leaves.mjs';
import {complianceMatrix, infeasibilityLedger, LEDGER_PATH} from '../../rust/tests/web/support/cnl-matrix.mjs';

const args = process.argv.slice(2);
const rows = complianceMatrix();
const detail = args.indexOf('--detail');
if (detail >= 0) {
  const [language, profile, density] = args.slice(detail + 1);
  for (const row of rows.filter(item => item.language === language && item.profile === profile && item.density === density && !item.feasible)) {
    console.log(row.graph, JSON.stringify(row.text), [...row.report.violations, ...row.report.unchecked].join(','));
  }
} else {
  const ledger = infeasibilityLedger(rows);
  const feasible = rows.filter(row => row.feasible).length;
  for (const [key, entry] of Object.entries(ledger)) console.log(key, entry.graphs, JSON.stringify(entry.failing));
  console.log(`feasible ${feasible}/${rows.length}`);
  if (args.includes('--write')) writeFileSync(LEDGER_PATH, compactJsonLeaves(JSON.stringify(ledger, null, 2) + '\n'));
}
