// Print the specification case ids extracted at a repository root, and the
// extraction refusals, so two checkouts can be diffed (issue #1632 gate check).
import { pathToFileURL } from 'node:url';
const root = process.argv[2] || process.cwd();
const { specificationCases } = await import(pathToFileURL(`${root}/scripts/lib/rust-specification-cases.mjs`).href);
const { cases, unsupported } = specificationCases(root);
for (const c of cases) console.log(`case ${c.id}`);
for (const u of unsupported) console.log(`refused ${u.id} :: ${u.reason}`);
