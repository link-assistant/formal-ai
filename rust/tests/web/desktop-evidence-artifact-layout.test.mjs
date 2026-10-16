import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import { execFileSync, spawnSync } from 'node:child_process';
import { createEvidenceFixture } from '../fixtures/native-release-evidence/observations.mjs';
import { RECEIPT_FILE } from '../../../scripts/native-release-artifact.mjs';
import { captureReleaseProtocol, verifyReleaseProtocol } from '../../../scripts/release-protocol.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const YAML = createRequire(import.meta.url)('yaml');
const workflow = YAML.parse(fs.readFileSync(path.join(root, '.github/workflows/desktop-release.yml'), 'utf8'));
const steps = Object.values(workflow.jobs).flatMap(job => job.steps ?? []);
const stage = steps.find(step => step.name === 'Stage distinct macOS signing observation');
const upload = steps.find(step => step.name === 'Upload actual macOS signing observation');
const download = steps.find(step => step.name === 'Download actual macOS signing observations');
const sourceUpload = workflow.jobs['native-source'].steps.find(step => step.with?.name === 'native-release-source');
function write(file, bytes) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, bytes);
}
function scratch(t) {
  const directory = fs.mkdtempSync(path.join(tmpdir(), 'desktop-evidence-layout-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}
function artifact(t, entry) {
  const directory = scratch(t);
  write(path.join(directory, 'signing-observation.json'), entry.bytes);
  if (stage) {
    const code = stage.run.match(/node --input-type=module - <<'NODE'\n([\s\S]+)\nNODE\s*$/u);
    assert.ok(code, 'actual workflow staging script required');
    const result = spawnSync(process.execPath, ['--input-type=module', '-'], {
      input: code[1], encoding: 'utf8', timeout: 5000,
      env: { ...process.env, RUNNER_TEMP: directory, NATIVE_SIGNING_LABEL: entry.label },
    });
    assert.equal(result.status, 0, result.stderr);
  }
  const selected = upload.with.path.replace('${{ runner.temp }}', directory);
  const stat = fs.statSync(selected);
  const files = stat.isFile() ? [path.basename(selected)]
    : fs.readdirSync(selected, { recursive: true }).filter(file => fs.statSync(path.join(selected, file)).isFile());
  const base = stat.isFile() ? path.dirname(selected) : selected;
  return { name: 'signing-' + entry.label, files: files.map(file => ({ file, bytes: fs.readFileSync(path.join(base, file)) })) };
}
function collect(t, labels) {
  const directory = scratch(t), fixture = createEvidenceFixture();
  const source = path.join(directory, 'source'), receipts = path.join(directory, 'receipts');
  const signing = path.join(directory, 'signing'), helpers = path.join(directory, 'helpers');
  write(path.join(source, 'source-selection.json'), fixture.sourceBytes);
  write(path.join(source, 'protocol-selection.json'), fixture.protocolBytes);
  for (const entry of fixture.receipts) write(path.join(receipts, 'native-receipt-' + entry.target, RECEIPT_FILE), entry.bytes);
  for (const [name, bytes] of Object.entries(fixture.helperBytes)) write(path.join(helpers, name), bytes);
  const artifacts = fixture.signing.filter(entry => labels.includes(entry.label)).map(entry => artifact(t, entry));
  // Actual download-artifact v8: a single pattern match is flattened even when
  // merge-multiple is false. Explicit merge plus internal label paths handles both counts.
  for (const entry of artifacts) for (const item of entry.files) {
    const parent = artifacts.length === 1 || download.with['merge-multiple'] === true ? signing : path.join(signing, entry.name);
    write(path.join(parent, item.file), item.bytes);
  }
  const b = fixture.bindings;
  const result = spawnSync(process.execPath, [path.join(root, 'scripts/native-release-evidence.mjs'), 'collect',
    source, receipts, signing, path.join(directory, 'output')], {
    encoding: 'utf8', timeout: 5000,
    env: { ...process.env, FORMAL_AI_NATIVE_PROTOCOL_DIR: helpers, NATIVE_SELECTION_SHA256: b.sourceSha256,
      NATIVE_SOURCE_COMMIT: b.sourceCommit, NATIVE_SOURCE_TREE: b.sourceTree, NATIVE_PACKAGE_VERSION: b.version,
      GITHUB_RUN_ID: b.run, NATIVE_COMPILER_RELEASE: b.compilerRelease, NATIVE_COMPILER_COMMIT: b.compilerCommit,
      NATIVE_PROTOCOL_SHA256: b.protocolSha256, NATIVE_PROTOCOL_COMMIT: b.protocolCommit },
  });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

test('one surviving signing producer stays correctly labeled while its missing peer remains a failure', t => {
  const result = collect(t, ['macos-x64']);
  assert.equal(result.complete, false);
  assert.deepEqual(result.failures.map(item => item.target), ['macos-arm64']);
});
test('two signing artifacts preserve distinct exact-byte records for the strict collector', t => {
  const result = collect(t, ['macos-arm64', 'macos-x64']);
  assert.equal(result.complete, true);
  assert.deepEqual(result.failures, []);
});
test('signing staging and explicit merge retain independent platform paths', () => {
  assert.ok(stage);
  assert.equal(download.with['merge-multiple'], true);
  assert.equal(upload.with.path, '${{ runner.temp }}/formal-ai-signing-artifact/');
});
test('default hidden exclusion reproduces lost advertisement; configured upload preserves strict protocol verification', t => {
  const directory = scratch(t), repository = path.join(directory, 'repository');
  fs.mkdirSync(repository);
  const definition = '.github/release-advertisement.json';
  write(path.join(repository, definition), fs.readFileSync(path.join(root, definition)));
  const git = (...args) => execFileSync('git', args, { cwd: repository, encoding: 'utf8', stdio: 'pipe' }).trim();
  git('init', '--initial-branch=main');git('config', 'user.name', 'Fixture');git('config', 'user.email', 'fixture@example.invalid');
  git('add', '.');git('commit', '-m', 'source-qualified fixture');const revision = git('rev-parse', 'HEAD');
  const original = path.join(directory, 'captured');
  const snapshot = captureReleaseProtocol(repository, original, revision, [definition]);
  function transport(name, includeHidden) {
    const destination = path.join(directory, name);
    fs.mkdirSync(destination);
    for (const relative of fs.readdirSync(original, { recursive: true })) {
      const physical = path.join(original, relative);
      if (!fs.statSync(physical).isFile()) continue;
      if (!includeHidden && relative.split(path.sep).some(part => part.startsWith('.'))) continue;
      write(path.join(destination, relative), fs.readFileSync(physical));
    }
    return destination;
  }
  const defaultUpload = transport('default', false);
  assert.throws(() => verifyReleaseProtocol(repository, defaultUpload, revision, snapshot.sha256), /ENOENT/u);
  const configured = transport('configured', sourceUpload.with['include-hidden-files'] === true);
  const verified = verifyReleaseProtocol(repository, configured, revision, snapshot.sha256);
  assert.equal(verified.sha256, snapshot.sha256);
  fs.appendFileSync(path.join(configured, definition), ' ');
  assert.throws(() => verifyReleaseProtocol(repository, configured, revision, snapshot.sha256), /differs from workflow source/u);
});
