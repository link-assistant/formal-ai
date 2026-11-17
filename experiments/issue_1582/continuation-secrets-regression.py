from pathlib import Path
path=Path('rust/tests/web/bounded-subprocess-boundaries.test.mjs')
source=path.read_text()
addition=r'''

test('secret scanner bounds npm shell arguments and retains every literal filename', {timeout: 60000}, async t => {
  const os = await import('node:os');
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-secret-batches-'));
  t.after(() => fs.rmSync(temporary, {recursive: true, force: true}));
  const receipts = path.join(temporary, 'receipts.jsonl');
  // npm turns each npx argument batch into one shell command. Model its
  // quoting cost without installing packages or exposing actual credentials.
  fs.writeFileSync(path.join(temporary, 'npx'), `#!${process.execPath}
const fs = require('node:fs');
const args = process.argv.slice(2);
const marker = args.indexOf('secretlint');
if (marker < 0) process.exit(2);
const files = args.slice(marker + 1);
const quote = value => "'" + value.replaceAll("'", "'" + String.fromCharCode(92) + "''") + "'";
const bytes = Buffer.byteLength('secretlint ' + files.map(quote).join(' '));
fs.appendFileSync(process.env.SECRET_SCAN_RECEIPTS, JSON.stringify({files, bytes}) + '\\n');
if (bytes > 120 * 1024) { console.error('mock npm spawn E2BIG'); process.exit(7); }
if (files.includes('fixture-detected-secret')) process.exit(1);
`, {mode: 0o755});
  const files = Array.from({length: 1800}, (_, index) =>
    `fixture/${index.toString().padStart(4, '0')}/a file with 'quotes' 世界 ${'a'.repeat(70)}.txt`);
  const run = argumentsValue => boundedCommand('bash',
    [path.join(root, 'scripts/check-secrets.sh'), ...argumentsValue], {
      cwd: root, timeout: 20000, maxBuffer: 1024 * 1024,
      env: {...process.env, PATH: `${temporary}${path.delimiter}${process.env.PATH}`,
        SECRET_SCAN_RECEIPTS: receipts},
    });
  const result = await run(files);
  assert.equal(result.error, undefined, result.stderr.toString());
  assert.equal(result.status, 0, result.stderr.toString());
  assert.equal(result.cleanup_complete, true);
  const batches = fs.readFileSync(receipts, 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.ok(batches.length > 1);
  assert.deepEqual(batches.flatMap(batch => batch.files), files);
  assert.ok(batches.every(batch => batch.bytes <= 120 * 1024));
  const finding = await run(['fixture-detected-secret']);
  assert.equal(finding.error, undefined);
  assert.notEqual(finding.status, 0, 'scanner findings must still fail the gate');
  assert.equal(finding.cleanup_complete, true);
});
'''
assert "test('secret scanner bounds" not in source
path.write_text(source+addition)
