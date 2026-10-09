import { strict as assert } from 'node:assert';

assert.deepEqual(process.argv.slice(2), ['diff', '--name-only', 'origin/main...HEAD']);
const mode = process.env.CHANGELOG_GIT_FIXTURE_MODE;
if (mode === 'failed') {
  process.stderr.write('fixture git failure\n');
  process.exitCode = 7;
} else if (mode === 'overflow') {
  process.stdout.write('x'.repeat(17 * 1024 * 1024));
} else if (mode === 'large') {
  const files = Array.from({ length: 11000 }, (_, index) =>
    `data/evidence/${String(index).padStart(5, '0')}/${'x'.repeat(100)}.lino`);
  files.push('rust/src/fixture.rs', 'changelog.d/fixture.md');
  process.stdout.write(`${files.join('\n')}\n`);
} else {
  assert.equal(mode, 'empty');
}
