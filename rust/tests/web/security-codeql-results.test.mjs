import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, readFileSync, chmodSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import YAML from 'yaml';

const cli = new URL('../../../scripts/check-codeql-results.mjs', import.meta.url);
const run = (results = []) => ({ tool: { driver: { name: 'CodeQL' } },
  invocations: [{ executionSuccessful: true }], results });
const sarif = (...runs) => ({ version: '2.1.0', runs });

test('SARIF gate rejects missing/invalid reports and checks every file/run/severity', { timeout: 20000 }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'codeql-results-'));
  const check = (expected) => {
    const result = spawnSync(process.execPath, [cli.pathname, dir], { encoding: 'utf8', timeout: 5000 });
    assert.equal(result.status === 0, expected, result.stdout + result.stderr);
    return result;
  };
  const write = (value, name = 'rust.sarif') => writeFileSync(join(dir, name),
    typeof value === 'string' ? value : JSON.stringify(value));
  try {
    check(false);
    for (const invalid of ['{', {}, sarif(), sarif({}), sarif({ ...run(), results: null }),
      sarif({ ...run(), invocations: [{ executionSuccessful: false }] }),
      sarif({ ...run(), invocations: [] }), sarif({ ...run(), results: [null] })]) {
      write(invalid); check(false);
    }
    write(sarif(run())); check(true);
    for (const level of ['error', 'warning', 'note', 'none', undefined]) {
      const finding = { ruleId: 'test/rule', message: { text: 'unresolved' }, level };
      write(sarif(run(), run([finding])));
      assert.match(check(false).stderr, /test\/rule/);
      write(sarif(run([{ ...finding, suppressions: [{ kind: 'inSource' }] }]))); check(false);
      write(sarif(run([{ ...finding, baselineState: 'absent' }]))); check(true);
    }
    write(sarif(run()));
    mkdirSync(join(dir, 'nested'));
    write(sarif(run([{ ruleId: 'test/second', message: { text: 'second file' } }])), 'nested/actions.sarif');
    check(false);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('SARIF invocation diagnostics gate extraction even with no findings', { timeout: 20000 }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'codeql-notifications-'));
  try {
    const check = (invocations) => {
      writeFileSync(join(dir, 'analysis.sarif'), JSON.stringify(sarif({ ...run(), invocations })));
      return spawnSync(process.execPath, [cli.pathname, dir], { encoding: 'utf8', timeout: 5000 });
    };
    for (const field of ['toolExecutionNotifications', 'toolConfigurationNotifications']) {
      const diagnostic = { descriptor: { id: 'extract/failure' }, message: { text: 'Extraction failed' },
        locations: [{ physicalLocation: { artifactLocation: { uri: 'src/lib.rs' }, region: { startLine: 42 } } }] };
      for (const level of ['error', 'warning', undefined]) {
        const result = check([{ executionSuccessful: true },
          { executionSuccessful: true, [field]: [{ ...diagnostic, level }] }]);
        assert.equal(result.status, 1);
        assert.match(result.stderr, /extract\/failure/);
        assert.match(result.stderr, /"src\/lib.rs":"42"/);
        assert.match(result.stderr, /Extraction failed/);
      }
      for (const value of [null, {}, 'invalid', [null], [1], [{}], [{ level: 'note' }],
        [{ message: { text: 'bad level' }, level: 'unknown' }],
        [{ message: 'invalid' }], [{ message: { text: 12 } }], [{ message: { text: 'null level' }, level: null }]]) {
        const result = check([{ executionSuccessful: true, [field]: value }]);
        assert.equal(result.status, 1, JSON.stringify(value));
        assert.match(result.stderr, /Malformed SARIF notification/);
      }
      for (const level of ['note', 'none']) {
        const result = check([{ executionSuccessful: true, [field]: [{ ...diagnostic, level }] }]);
        assert.equal(result.status, 0, result.stderr);
      }
      assert.equal(check([{ executionSuccessful: true, [field]: [] }]).status, 0);
    }
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('SARIF collects malformed items, diagnostics and findings across files without workflow commands', { timeout: 10000 }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'codeql-all-problems-'));
  const payload = 'x\n::error title=spoof::fake\r::warning::fake';
  try {
    writeFileSync(join(dir, 'a.sarif'), '{');
    writeFileSync(join(dir, 'b.sarif'), JSON.stringify(sarif({ ...run([
      null, { ruleId: payload, level: payload, message: { text: payload },
        locations: [{ physicalLocation: { artifactLocation: { uri: payload }, region: { startLine: payload } } }] },
    ]), invocations: [{ executionSuccessful: false, toolExecutionNotifications: [null,
      { descriptor: { id: payload }, level: 'error', message: { text: 'diagnostic' } }],
      toolConfigurationNotifications: {} }] })));
    writeFileSync(join(dir, 'c.sarif'), JSON.stringify(sarif(run([
      { ruleId: 'malformed/location', message: { text: 'malformed object' },
        locations: [{ physicalLocation: { artifactLocation: { uri: { toString: null } },
          region: { startLine: { toString: null } } } }] },
      { ruleId: 'last/finding', message: { text: 'last file' } },
    ]))));
    const result = spawnSync(process.execPath, [cli.pathname, dir], { encoding: 'utf8', timeout: 5000 });
    assert.equal(result.status, 1);
    for (const text of ['Invalid SARIF report', 'Malformed SARIF notification:',
      'Malformed SARIF notification array', 'Malformed SARIF result', 'diagnostic', 'last/finding',
      'Missing or unsuccessful', JSON.stringify(payload), JSON.stringify(JSON.stringify({ toString: null }))]) assert.ok(result.stderr.includes(text), text);
    assert.doesNotMatch(result.stderr, /^::/m);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

const security = () => YAML.parse(readFileSync('.github/workflows/security.yml', 'utf8')).jobs.codeql;

test('the CodeQL job checks and preserves the exact reports produced by analysis', () => {
  const job = security();
  const analysis = job.steps.find(step => step.uses?.startsWith('github/codeql-action/analyze@'));
  const gate = job.steps.find(step => step.name === 'Reject unresolved CodeQL findings');
  const archive = job.steps.find(step => step.name === 'Preserve CodeQL reports');
  assert.ok(analysis?.with?.output, 'analysis must write an explicit report directory');
  assert.ok(gate, 'successful upload alone does not establish absence of findings');
  assert.equal(gate.if, '${{ !cancelled() }}');
  assert.equal(gate.run, `node scripts/check-codeql-results.mjs "${analysis.with.output}"`);
  assert.ok(job.steps.indexOf(gate) > job.steps.indexOf(analysis));
  assert.equal(archive.if, '${{ !cancelled() }}');
  assert.equal(archive.with.path, analysis.with.output + '/');
  assert.equal(archive.with['if-no-files-found'], 'error');
  assert.ok(job.steps.indexOf(archive) > job.steps.indexOf(gate));
  assert.ok(job.steps.some(step => step.uses?.startsWith('actions/setup-node@')));
});

for (const [label, installStatus, sysrootStatus, sourcePresent, success] of [
  ['unavailable toolchain', 1, 0, true, false],
  ['failed sysroot discovery', 0, 1, true, false],
  ['missing standard-library sources', 0, 0, false, false],
  ['verified sysroot and sources', 0, 0, true, true],
]) {
  test('the actual CodeQL pin shell refuses incomplete coverage: ' + label, { timeout: 10000 }, () => {
    const dir = mkdtempSync(join(tmpdir(), 'codeql-pin-'));
    try {
      const sysroot = join(dir, 'sysroot'), environment = join(dir, 'environment');
      const source = join(sysroot, 'lib/rustlib/src/rust/library');
      if (sourcePresent) mkdirSync(source, { recursive: true });
      writeFileSync(environment, '');
      const rustup = join(dir, 'rustup');
      writeFileSync(rustup, '#!/bin/sh\nif [ "$1" = toolchain ]; then exit "$FIXTURE_INSTALL_STATUS"; fi\nprintf "%s\\n" "$FIXTURE_SYSROOT"\nexit "$FIXTURE_SYSROOT_STATUS"\n');
      chmodSync(rustup, 0o755);
      const step = security().steps.find(item => item.name === 'Pin the extractor sysroot to a std the CodeQL bundle can parse');
      const result = spawnSync('bash', ['-c', step.run], { encoding: 'utf8', timeout: 5000,
        env: { ...process.env, PATH: dir + ':' + process.env.PATH, GITHUB_ENV: environment,
          CODEQL_RUST_SYSROOT_TOOLCHAIN: 'fixture', FIXTURE_INSTALL_STATUS: String(installStatus),
          FIXTURE_SYSROOT_STATUS: String(sysrootStatus), FIXTURE_SYSROOT: sysroot } });
      assert.equal(result.signal, null, result.stderr);
      assert.equal(result.status === 0, success, result.stdout + result.stderr);
      const exported = readFileSync(environment, 'utf8');
      if (success) assert.equal(exported, `CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT=${sysroot}\nCODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC=${source}\n`);
      else assert.equal(exported, '', 'failed prerequisites must never export a partial sysroot');
    } finally { rmSync(dir, { recursive: true, force: true }); }
  });
}
