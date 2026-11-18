// Lists the requirements whose ledger test runs only in Rust (R1188-U29) for
// the given checkout, so the ones this branch adds can be pinned by a
// JavaScript test. Usage: node rust-only-pins.mjs [ROOT]
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { ledgerRecords, rootOf } from '../../scripts/render-progressive-plan.mjs';

const root = process.argv[2] ?? '.';
const ledger = join(root, 'data/meta/requirement-status-ledger');
for (const name of readdirSync(ledger).filter((file) => file.endsWith('.lino')).sort()) {
  for (const record of ledgerRecords(readFileSync(join(ledger, name), 'utf8'))) {
    if (record.id && rootOf(record) === 'rust') console.log(`${record.id}\t${record.verdict}\t${record.test}`);
  }
}
