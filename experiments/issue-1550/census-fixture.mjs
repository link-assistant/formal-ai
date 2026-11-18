// Issue #1550 (acceptance criterion c): write (or check) the procedure census
// fixture rust/tests/fixtures/procedure-census-prompts.json, the prompts of
// every specification test that scripts/lib/rust-specification-cases.mjs
// carries as an ask. The native census rust/tests/unit/meta_executor_census.rs
// solves each and requires the recursive procedure's `meta_exec:result` in the
// turn's trace; rust/tests/web/issue-1550-procedure-census.test.mjs checks that
// the fixture is current.
// Usage: node experiments/issue-1550/census-fixture.mjs --write|--check
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { censusFixtureText } from '../../rust/tests/web/helpers/issue-1550-procedure-census.mjs';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const FIXTURE = path.join(REPO, 'rust/tests/fixtures/procedure-census-prompts.json');
const text = censusFixtureText(REPO);
if (process.argv.includes('--write')) {
  fs.writeFileSync(FIXTURE, text);
  console.log(`wrote ${path.relative(REPO, FIXTURE)} with ${JSON.parse(text).prompts.length} prompts`);
} else if (!fs.existsSync(FIXTURE) || fs.readFileSync(FIXTURE, 'utf8') !== text) {
  console.error('procedure census fixture is stale; run with --write');
  process.exitCode = 1;
} else {
  console.log(`procedure census fixture: ${JSON.parse(text).prompts.length} prompts current`);
}
