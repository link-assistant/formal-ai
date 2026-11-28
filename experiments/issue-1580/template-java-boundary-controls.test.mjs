// Execute exact pinned release helpers; model workflow guards without publication.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {copyFileSync, existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {Script} from 'node:vm';
import YAML from 'yaml';

const fixture = new URL('./fixtures/java-template/', import.meta.url);
const provenance = JSON.parse(readFileSync(new URL('source.json', fixture), 'utf8'));
for (const row of provenance.files) {
  const bytes = readFileSync(new URL(row.path, fixture));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'), row.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), row.sha256);
}
const jobs = YAML.parse(readFileSync(new URL('.github/workflows/release.yml', fixture), 'utf8')).jobs;

// GitHub's implicit success() remains active unless a status function is present:
// https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#status-check-functions
function admitted(job, {event = 'workflow_dispatch', results = {}, outputs = {}, mode = 'instant', cancelled = false} = {}) {
  const needs = Object.fromEntries((job.needs ?? []).map(name => [name,
    {result: results[name] ?? 'success', outputs: outputs[name] ?? {}}]));
  const source = String(job.if ?? 'true').trim().replace(/^\$\{\{\s*|\s*\}\}$/gu, '')
    .replace(/\.([\w]+-[\w-]+)/gu, (_, name) => `['${name}']`);
  const success = () => !cancelled && Object.values(needs).every(value => value.result === 'success');
  const explicitStatus = /\b(?:always|success|failure|cancelled)\s*\(/u.test(source);
  const value = new Script(source).runInNewContext({
    github: {event_name: event, event: {inputs: {release_mode: mode}}}, needs,
    always: () => true, cancelled: () => cancelled, success,
    failure: () => Object.values(needs).some(value => value.result === 'failure'),
  }, {timeout: 1000});
  return Boolean(value) && (explicitStatus || success());
}

const lintCandidate = process.env.ISSUE_1580_JAVA_KEEP_ORIGINAL_DISPATCH_GUARD === '1' ? jobs.lint : {
  ...jobs.lint,
  if: `!cancelled() && (github.event_name == 'workflow_dispatch' ||
    (needs.detect-changes.result == 'success' && (${jobs.lint.if.trim()})))`,
};
const buildCandidate = process.env.ISSUE_1580_JAVA_KEEP_ORIGINAL_DISPATCH_GUARD === '1' ? jobs.build : {
  ...jobs.build,
  if: `!cancelled() && needs.test.result == 'success' &&
    (needs.lint.result == 'success' || (github.event_name != 'workflow_dispatch' && needs.lint.result == 'skipped'))`,
};

test('Java manual dispatch skips its lint prerequisite in the exact pinned workflow model', {timeout: 10000}, () => {
  assert.equal(admitted(jobs['detect-changes']), false);
  assert.equal(admitted(jobs.lint, {results: {'detect-changes': 'skipped'}}), false);
  assert.equal(admitted(jobs.test, {results: {'detect-changes': 'skipped', 'changeset-check': 'skipped'}}), true);
  assert.equal(admitted(jobs.build, {results: {lint: 'skipped', test: 'success'}}), true);
  for (const mode of ['instant', 'changeset']) {
    const release = jobs[`manual-release-${mode}`];
    assert.equal(admitted(release, {mode}), true);
    for (const result of ['failure', 'cancelled', 'skipped']) {
      assert.equal(admitted(release, {mode, results: {build: result}}), false);
    }
  }
});

test('Java dispatch candidate runs lint and requires both quality jobs to succeed', {timeout: 10000}, () => {
  assert.equal(admitted(lintCandidate, {results: {'detect-changes': 'skipped'}}), true);
  assert.equal(admitted(buildCandidate), true);
  for (const name of ['lint', 'test']) {
    for (const result of ['failure', 'cancelled', 'skipped']) {
      assert.equal(admitted(buildCandidate, {results: {[name]: result}}), false, `${name}: ${result}`);
    }
  }
  assert.equal(admitted(lintCandidate, {cancelled: true}), false);
  assert.equal(admitted(buildCandidate, {cancelled: true}), false);
});

test('Java dispatch candidate preserves push and pull-request change and failure controls', {timeout: 10000}, () => {
  for (const event of ['push', 'pull_request']) {
    assert.equal(admitted(lintCandidate, {event}), admitted(jobs.lint, {event}));
    for (const result of ['failure', 'cancelled', 'skipped']) {
      assert.equal(admitted(lintCandidate, {event, results: {'detect-changes': result}}), false);
    }
    assert.equal(admitted(buildCandidate, {event, results: {lint: 'skipped'}}), true);
    for (const flag of ['java', 'pom', 'mjs', 'docs', 'workflow']) {
      const context = {event, outputs: {'detect-changes': {[`${flag}-changed`]: 'true'}}};
      assert.equal(admitted(jobs.lint, context), true);
      assert.equal(admitted(lintCandidate, context), true);
    }
  }
});

const collector = readFileSync(new URL('scripts/collect-changelog.mjs', fixture), 'utf8');
const readStatement = "const content = readFileSync(filePath, 'utf-8');";
assert.equal(collector.split(readStatement).length, 2);
const fragmentCandidate = process.env.ISSUE_1580_JAVA_KEEP_ORIGINAL_FRAGMENT_READER === '1' ? collector : collector
  .replace(readStatement, "const content = readFileSync(filePath, 'utf-8').replace(/\\r\\n/g, '\\n');")
  .replace('    if (description) {',
    "    if (!description) throw new Error('Invalid or empty changeset: ' + basename(path));\n    if (description) {");
const baseline = '# Changelog\n\n<!-- changelog-insert-here -->\n\nExisting release notes.\n';
const validNote = '---\n"my-package": patch\n---\nPreserved real release note.\n';
const fragmentCases = [
  {name: 'valid LF', text: validNote, valid: true},
  {name: 'valid CRLF', text: validNote.replaceAll('\n', '\r\n'), valid: true},
  {name: 'missing frontmatter', text: 'Preserved real release note.\n', valid: false},
  {name: 'empty description', text: '---\n"my-package": patch\n---\n\n', valid: false},
  {name: 'mixed valid and malformed', text: 'Invalid release note.\n', valid: false, extra: validNote},
];

function executeCollector(source, item) {
  const directory = mkdtempSync(join(tmpdir(), 'issue-1580-java-fragments-'));
  try {
    mkdirSync(join(directory, 'scripts'));
    mkdirSync(join(directory, '.changeset'));
    writeFileSync(join(directory, 'scripts/collect-changelog.mjs'), source);
    copyFileSync(new URL('scripts/github-actions-log.mjs', fixture), join(directory, 'scripts/github-actions-log.mjs'));
    writeFileSync(join(directory, 'CHANGELOG.md'), baseline);
    writeFileSync(join(directory, '.changeset/note.md'), item.text);
    writeFileSync(join(directory, '.changeset/README.md'), 'Retain this guide.\n');
    if (item.extra) writeFileSync(join(directory, '.changeset/good.md'), item.extra);
    const result = spawnSync(process.execPath, ['scripts/collect-changelog.mjs', '--version', '1.2.4'], {
      cwd: directory, encoding: 'utf8', maxBuffer: 65536,
    });
    assert.equal(result.error, undefined);
    const exists = existsSync(join(directory, '.changeset/note.md'));
    const extraExists = existsSync(join(directory, '.changeset/good.md'));
    assert.equal(readFileSync(join(directory, '.changeset/README.md'), 'utf8'), 'Retain this guide.\n');
    return {exitCode: result.status, stdout: result.stdout, stderr: result.stderr,
      changelog: readFileSync(join(directory, 'CHANGELOG.md'), 'utf8'),
      fragment: exists ? readFileSync(join(directory, '.changeset/note.md'), 'utf8') : null,
      extra: extraExists ? readFileSync(join(directory, '.changeset/good.md'), 'utf8') : null};
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
}

test('Java original collector deletes skipped CRLF, malformed and empty release fragments', {timeout: 10000}, () => {
  for (const item of fragmentCases) {
    const result = executeCollector(collector, item);
    assert.equal(result.exitCode, 0, result.stderr);
    assert.equal(result.fragment, null, item.name);
    assert.equal(result.extra, null);
    assert.equal(result.changelog.includes('Preserved real release note.'), item.name === 'valid LF' || Boolean(item.extra));
  }
});

test('Java collector candidate retains valid notes and preserves complete rejected batches', {timeout: 10000}, () => {
  for (const item of fragmentCases) {
    const result = executeCollector(fragmentCandidate, item);
    if (item.valid) {
      assert.equal(result.exitCode, 0, result.stderr);
      assert.equal(result.fragment, null, item.name);
      assert(result.changelog.includes('Preserved real release note.'), item.name);
    } else {
      assert.notEqual(result.exitCode, 0, item.name);
      assert.equal(result.fragment, item.text, item.name);
      assert.equal(result.changelog, baseline, item.name);
      assert.equal(result.extra, item.extra ?? null, item.name);
    }
  }
});
