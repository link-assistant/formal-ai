// Static documentation consistency observation; no Cargo command is executed.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import test from 'node:test';
import YAML from 'yaml';

const directory = new URL('./fixtures/rust-timeout-documentation/', import.meta.url);
const manifest = JSON.parse(readFileSync(new URL('manifest.json', directory), 'utf8'));
function source(path) {
  const row = manifest.sources.find(row => row.path === path);
  const packed = readFileSync(new URL(row.file, directory));
  assert.equal(createHash('sha256').update(packed).digest('hex'), row.archiveSha256);
  const raw = gunzipSync(packed);
  assert.equal(raw.length, row.bytes);
  assert.equal(createHash('sha256').update(raw).digest('hex'), row.sha256);
  assert.equal(createHash('sha1').update(Buffer.from('blob ' + raw.length + '\0')).update(raw).digest('hex'), row.gitBlob);
  return raw.toString('utf8');
}
const readme = source('README.md');
const workflow = source('.github/workflows/release.yml');
const job = YAML.parse(workflow).jobs.test;
const testStep = job.steps.find(step => step.name === 'Run tests');
const actualMinutes = job['timeout-minutes'];
const budgetSeconds = testStep.env.TEST_BUDGET_SECONDS;
const declaration = /CI caps each test-matrix job at (\d+) minutes\./;

test('pinned Rust README cap disagrees with the actual matrix job', () => {
  const documentedMinutes = Number(readme.match(declaration)?.[1]);
  assert.deepEqual({documentedMinutes, actualMinutes, budgetSeconds}, {documentedMinutes: 10, actualMinutes: 30, budgetSeconds: 900});
  assert.notEqual(documentedMinutes, actualMinutes);
  assert.match(testStep.run, /run-with-budget-warning\.sh "\$TEST_BUDGET_SECONDS" "Test suite"/);
});

test('disposable wording candidate distinguishes the backstop and internal budget', () => {
  const candidate = readme.replace(declaration, 'CI gives each test-matrix job a 30-minute backstop and the test step a 900-second (15-minute) execution budget.');
  assert.notEqual(candidate, readme);
  assert.equal(Number(candidate.match(/a (\d+)-minute backstop/)?.[1]), actualMinutes);
  assert.equal(Number(candidate.match(/a (\d+)-second \(15-minute\)/)?.[1]), budgetSeconds);
  assert.match(candidate, /cargo test` does not provide a portable global per-test timeout/);
  assert.ok(!declaration.test(candidate));
});
