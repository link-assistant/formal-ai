// Check every generalized CNL property for one language grammar only (issue #1584).
// Usage: node experiments/issue-1584/check-language.mjs <language>[,<language>...] [--ledger]
// Other languages' grammar files are hidden, so one draft cannot break another;
// a comma list checks those languages together (cross-language prompt analysis).
const languages = process.argv[2].split(',');
process.env.CNL_MATRIX_LANGUAGES = [...new Set(['en', ...languages])].join(',');
const matrix = await import('../../rust/tests/web/support/cnl-matrix.mjs');
let total = 0;
for (const language of languages) {
  for (const [property, list] of Object.entries(matrix.languageProperties(language))) {
    total += list.length;
    console.log(`${language} ${property}: ${list.length}`);
    for (const line of list.slice(0, 15)) console.log('  ' + line);
  }
}
const rows = matrix.complianceMatrix(languages);
const failures = matrix.complianceFailures(rows);
total += failures.length;
console.log(`compliance: ${failures.length}`);
for (const line of failures.slice(0, 25)) console.log('  ' + line);
if (process.argv.includes('--ledger')) {
  for (const [key, entry] of Object.entries(matrix.infeasibilityLedger(rows))) console.log(key, entry.graphs, JSON.stringify(entry.failing));
}
console.log(total ? `FAIL ${total}` : 'PASS');
process.exitCode = total ? 1 : 0;
