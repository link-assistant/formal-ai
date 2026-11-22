import { strict as assert } from 'node:assert';

const addedOnly = process.argv.includes('--diff-filter=A');
assert.deepEqual(process.argv.slice(2), ['diff', ...(addedOnly ? ['--find-renames=100%', '--diff-filter=A'] : ['--no-renames']), '--name-only', '-z', 'origin/main...HEAD']);
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
  process.stdout.write(`${(addedOnly ? ['changelog.d/fixture.md'] : files).join('\0')}\0`);
} else if (mode === 'invalid-utf8') {
  process.stdout.write(Buffer.from([0xff, 0]));
} else if (mode === 'unterminated') {
  process.stdout.write('rust/src/unclosed.rs');
} else if (mode === 'empty-record') {
  process.stdout.write('rust/src/ok.rs\0\0');
} else {
  assert.equal(mode, 'empty');
}
