import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, writeFileSync, readFileSync, readdirSync, realpathSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import {createHash} from 'node:crypto';
import {prepareDraftEvidence} from '../../../scripts/prepare-hive-draft-evidence.mjs';

const wrapper = new URL('../../../desktop/scripts/package-macos-with-retry.sh', import.meta.url).pathname;
const workflow = readFileSync(new URL('../../../.github/workflows/desktop-release.yml', import.meta.url), 'utf8');
function fixture(t, {mode = 'success', brokenCapture = false} = {}) {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-packaging-console-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  const bin = join(root, 'bin'), runner = join(root, 'runner temporary');
  mkdirSync(bin); mkdirSync(runner); mkdirSync(join(root, 'release'));
  const executable = (name, source) => writeFileSync(join(bin, name), '#!/bin/bash\n' + source, {mode: 0o755});
  executable('sync', 'exit 0\n');
  executable('npx', `set -eu
n=0
if [ -f attempts ]; then n="$(cat attempts)"; fi
n=$((n + 1)); echo "$n" > attempts
printf '%s\\n' "$*"
for ((i=1; i<=240; i++)); do echo "attempt-$n-stdout-$i"; done
echo "attempt-$n-stderr" >&2
case "$MODE" in
  transient) if [ "$n" -eq 1 ]; then echo 'hdiutil: create failed - Device not configured' >&2; exit 37; fi ;;
  persistent) echo 'hdiutil: create failed - Device not configured' >&2; exit 37 ;;
  signing) echo 'codesign: errSecInternalComponent' >&2; exit 42 ;;
esac
`);
  if (brokenCapture) executable('tee', 'cat >/dev/null\nexit 7\n');
  const output = spawnSync('bash', [wrapper, '--mac', '--publish', 'never'], {
    cwd: root, timeout: 8000, maxBuffer: 1024 * 1024, encoding: 'utf8',
    env: {...process.env, PATH: bin + ':' + process.env.PATH, RUNNER_TEMP: runner,
      MODE: mode, FORMAL_AI_MACOS_PACKAGE_RETRY_DELAY_SECONDS: '0',
      FORMAL_AI_MACOS_PACKAGE_BUDGET_SECONDS: '0', FORMAL_AI_MACOS_PACKAGE_DEADLINE_EPOCH: ''},
  });
  assert.ifError(output.error);
  const directory = join(runner, 'formal-ai-packaging-logs');
  return {output, root, directory, outcome: n => JSON.parse(readFileSync(join(directory, `attempt-${n}.json`), 'utf8')),
    log: n => readFileSync(join(directory, `attempt-${n}.log`), 'utf8')};
}

test('packaging retains full stdout and stderr including lines omitted by a 200-line tail', t => {
  const f = fixture(t);
  assert.equal(f.output.status, 0);
  assert.match(f.log(1), /^--no-install electron-builder --mac --publish never\n/);
  assert.match(f.log(1), /attempt-1-stdout-1\n/);
  assert.match(f.log(1), /attempt-1-stdout-240\n/);
  assert.match(f.log(1), /attempt-1-stderr\n/);
  assert.deepEqual({...f.outcome(1), durationSeconds: 0}, {
    state: 'finished', attempt: 1, exitCode: 0, logExitCode: 0, durationSeconds: 0, consoleComplete: true,
  });
});
test('successful retry retains the failed attempt and both original command outcomes', t => {
  const f = fixture(t, {mode: 'transient'});
  assert.equal(f.output.status, 0);
  assert.match(f.log(1), /hdiutil: create failed - Device not configured/);
  assert.match(f.log(2), /attempt-2-stderr/);
  assert.equal(f.outcome(1).exitCode, 37);
  assert.equal(f.outcome(2).exitCode, 0);
  assert.equal(f.outcome(1).consoleComplete, true);
});
test('persistent hdiutil failure retains all three attempts and remains a failure', t => {
  const f = fixture(t, {mode: 'persistent'});
  assert.equal(f.output.status, 37);
  assert.deepEqual(readdirSync(f.directory).sort(), [1, 2, 3].flatMap(n => [`attempt-${n}.json`, `attempt-${n}.log`]));
  for (const n of [1, 2, 3]) {
    assert.match(f.log(n), new RegExp(`attempt-${n}-stdout-1\\n`));
    assert.equal(f.outcome(n).exitCode, 37);
    assert.equal(f.outcome(n).consoleComplete, true);
  }
});
test('real signing defects fail on the first attempt with their complete diagnostic', t => {
  const f = fixture(t, {mode: 'signing'});
  assert.equal(f.output.status, 42);
  assert.match(f.log(1), /codesign: errSecInternalComponent/);
  assert.equal(f.outcome(1).exitCode, 42);
  assert.equal(readdirSync(f.directory).length, 2);
});
for (const [mode, status] of [['success', 7], ['persistent', 37]]) {
  test(`capture failure is explicit and preserves the ${mode} command status`, t => {
    const f = fixture(t, {mode, brokenCapture: true});
    assert.equal(f.output.status, status);
    assert.equal(f.outcome(1).logExitCode, 7);
    assert.equal(f.outcome(1).consoleComplete, false);
    assert.equal(f.outcome(1).exitCode, mode === 'success' ? 0 : 37);
    assert.equal(readdirSync(f.directory).length, 1);
    assert.match(f.output.stderr, /capture failed/);
  });
}
test('workflow uploads packaging consoles even after failure and before smoke tests or post hooks', () => {
  const marker = workflow.indexOf('name: Upload complete packaging console');
  assert.ok(marker > 0 && marker < workflow.indexOf('      - name: Normalize desktop artifact names'));
  const upload = workflow.slice(marker, workflow.indexOf('\n', marker));
  for (const required of ['always()', 'actions/upload-artifact@v7', 'packaging-console-${{ matrix.label }}',
    '${{ runner.temp }}/formal-ai-packaging-artifact/', 'retention-days: 30']) assert.ok(upload.includes(required));
  assert.match(workflow, /os: macos-15,\s+ebflag: "--mac",\s+label: "macos-arm64"/);
  assert.match(workflow, /\[macos-arm64\]="macos-15"/);
  // The native producer and CLI keep their original deployment SDK and selected-source identity checks.
  assert.match(workflow, /os: macos-14,\s+target: aarch64-apple-darwin/);
  assert.ok(workflow.includes('required_macos_artifacts=('));
  assert.ok(workflow.includes('codesign --verify'));
  assert.match(workflow, /RUST_BACKTRACE: full/);
  const reusable = readFileSync(new URL('../../../.github/workflows/agent-cli-e2e.yml', import.meta.url), 'utf8');
  assert.match(reusable, /RUST_BACKTRACE: full/);
});
test('Hive integration puts scratch and child temporary files on the runner workspace without lowering admission', t => {
  const source = readFileSync(new URL('../../../experiments/issue_921_hive_mind_full_circle/run.sh', import.meta.url), 'utf8');
  const setup = source.match(/^SCRATCH=.*\nexport TMPDIR="\$\{SCRATCH\}"/m)?.[0];
  assert.ok(setup);
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-hive-temp-placement-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  const runner = join(root, 'workspace'), small = join(root, 'small-tmp');
  mkdirSync(runner); mkdirSync(small);
  assert.ok(source.includes('  cd "${SCRATCH}"\n'));
  assert.ok(source.includes('-name \'solve-*.log\''));
  assert.ok(source.indexOf('-exec cp -- {} "${OUT}/raw-logs/"') < source.indexOf('  rm -rf "${SCRATCH}"'));
  const result = spawnSync('bash', ['-eu', '-c', setup + '\ncd "$SCRATCH"\n' + 'node -p "JSON.stringify({cwd:process.cwd(),tmp:require(\'node:os\').tmpdir()})"'], {
    encoding: 'utf8', timeout: 5000, env: {...process.env, RUNNER_TEMP: runner, TMPDIR: small},
  });
  assert.ifError(result.error); assert.equal(result.status, 0, result.stderr);
  const placement = JSON.parse(result.stdout);
  assert.equal(placement.cwd, realpathSync(placement.tmp));
  assert.ok(placement.cwd.startsWith(realpathSync(runner) + '/formal-ai-issue-921.'));
  assert.ok(!source.includes('--min-disk-space'));
  assert.ok(source.includes('test "${PREPARE_STATUS}" -eq 0'));
  assert.ok(source.includes('"${RUNNER}" "${HIVE_MIND_ROOT}"'));
});

test('Hive failure cleanup retains the complete solve log and the original failing status', t => {
  const source = readFileSync(new URL('../../../experiments/issue_921_hive_mind_full_circle/run.sh', import.meta.url), 'utf8');
  const cleanup = source.slice(source.indexOf('cleanup() {'), source.indexOf('\nmkdir -p \\\n'));
  assert.ok(cleanup.includes('trap'));
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-hive-cleanup-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  const scratch = join(root, 'scratch'), out = join(root, 'evidence');
  mkdirSync(scratch); mkdirSync(join(out, 'raw-logs'), {recursive: true});
  const complete = Array.from({length: 240}, (_, n) => `diagnostic-${n}`).join('\n') + '\n';
  writeFileSync(join(scratch, 'solve-failure.log'), complete);
  const result = spawnSync('bash', ['-euo', 'pipefail', '-c', cleanup + '\nexit 37'], {
    timeout: 5000, encoding: 'utf8', env: {...process.env, SCRATCH: scratch, OUT: out, SERVER_PID: ''},
  });
  assert.ifError(result.error);
  assert.equal(result.status, 37);
  assert.match(result.stderr, /Hive Mind full-circle failed at run.sh:[0-9]+ \(exit 37\)/);
  assert.equal(readFileSync(join(out, 'raw-logs', 'solve-failure.log'), 'utf8'), complete);
  assert.ok(!readdirSync(root).includes('scratch'));
});

test('packaging artifact staging preserves complete sanitized logs and hashes while excluding raw credentials', t => {
  const f = fixture(t, {mode: 'transient'});
  const certificate = 'synthetic-signing-certificate-payload-xyz';
  const password = 'synthetic-signing-password-xyz';
  const original = 'BEGIN\n' + 'detail α😀\n'.repeat(140000) +
    `certificate=${certificate}\npassword=${password}\ncontact=fixture@example.invalid\n/home/private-user/work\nEND\n`;
  writeFileSync(join(f.directory, 'attempt-1.log'), original);
  const destination = join(f.root, 'sanitized-artifact');
  const receipt = prepareDraftEvidence({source: realpathSync(f.directory), destination,
    env: {MAC_SIGNING_CERTIFICATE_SECRET: certificate, MAC_SIGNING_PASSWORD_SECRET: password}});
  assert.equal(receipt.complete, true);
  assert.equal(receipt.rawPrivateFilesIncluded, false);
  assert.equal(receipt.taskAcceptance, false);
  const retained = readFileSync(join(destination, 'attempt-1.log'), 'utf8');
  assert.equal(retained, original.replace(certificate, '[REDACTED]').replace(password, '[credential]')
    .replace('fixture@example.invalid', '[email]').replace('/home/private-user', '/home/[user]'));
  const entry = receipt.files.find(row => row.path === 'attempt-1.log');
  assert.equal(entry.bytes, Buffer.byteLength(retained));
  assert.equal(entry.sha256, createHash('sha256').update(retained).digest('hex'));
  assert.equal(entry.complete, true);
  assert.equal(JSON.parse(readFileSync(join(destination, 'attempt-1.json'), 'utf8')).exitCode, 37);
  const stage = workflow.indexOf('name: Stage redacted packaging console');
  const upload = workflow.indexOf('name: Upload complete packaging console');
  assert.ok(stage > 0 && stage < upload);
  const staging = workflow.slice(stage, workflow.indexOf('\n', stage));
  for (const required of ['always()', 'scripts/prepare-hive-draft-evidence.mjs',
    'MAC_SIGNING_CERTIFICATE_SECRET', 'MAC_SIGNING_PASSWORD_SECRET']) assert.ok(staging.includes(required));
  assert.ok(!workflow.slice(upload, workflow.indexOf('\n', upload)).includes('/formal-ai-packaging-logs/'));
});
