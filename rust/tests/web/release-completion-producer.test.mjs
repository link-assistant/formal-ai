import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { productionProducerProfile, validateProducerObservation, joinProductionProducerEvidence } from '../../../scripts/lib/release-completion-producer.mjs';
const fixture = mkdtempSync(join(tmpdir(), 'pr1188-producer-contract-'));
const helper = fileURLToPath(new URL('../../../scripts/lib/release-completion-producer.mjs', import.meta.url));
const git = args => execFileSync('git', args, {
  cwd: fixture,
  stdio: 'pipe'
}).toString().trim();
mkdirSync(join(fixture, '.github/workflows'), {
  recursive: true
});
mkdirSync(join(fixture, 'scripts/lib'), {
  recursive: true
});
writeFileSync(join(fixture, 'scripts/lib/release-completion-producer.mjs'), readFileSync(helper));
writeFileSync(join(fixture, '.github/workflows/publication.yml'), 'name: Fixture\non: push\njobs:\n  publish:\n    name: Source publisher\n    steps:\n      - name: Publish original product\n        run: echo source-fixture-only\n      - name: Check actual registry and runtime\n        run: echo source-fixture-only\n');
git(['init', '-q']);
git(['config', 'user.email', 'source-fixture@example.invalid']);
git(['config', 'user.name', 'Source fixture']);
git(['remote', 'add', 'origin', 'https://github.com/example/project.git']);
git(['add', '.']);
git(['commit', '-qm', 'source-only fixture']);
const head = git(['rev-parse', 'HEAD']);
const profile = productionProducerProfile(fixture, head, '.github/workflows/publication.yml', 'publish');
const observed = () => ({
  defaultBranch: 'trunk',
  run: {
    id: 1,
    run_attempt: 2,
    status: 'completed',
    event: 'push',
    head_branch: 'trunk',
    head_sha: head,
    workflow_id: 3,
    path: '.github/workflows/publication.yml',
    repository: {
      full_name: 'example/project'
    },
    head_repository: {
      full_name: 'example/project'
    }
  },
  attempt: {
    id: 1,
    run_attempt: 2,
    status: 'completed',
    event: 'push',
    head_branch: 'trunk',
    head_sha: head,
    workflow_id: 3,
    path: '.github/workflows/publication.yml',
    repository: {
      full_name: 'example/project'
    },
    head_repository: {
      full_name: 'example/project'
    }
  },
  workflow: {
    id: 3,
    path: '.github/workflows/publication.yml'
  },
  job: {
    id: 4,
    run_id: 1,
    head_sha: head,
    name: 'Source publisher',
    status: 'completed',
    conclusion: 'success',
    steps: [{
      name: 'Publish original product',
      status: 'completed',
      conclusion: 'success'
    }, {
      name: 'Check actual registry and runtime',
      status: 'completed',
      conclusion: 'success'
    }]
  }
});
const refuse = (name, change) => test(name, () => {
  const input = observed();
  change(input);
  assert.throws(() => validateProducerObservation(profile, input));
});
test('complete structural fixture cannot mint authenticated delivery', () => {
  const checked = validateProducerObservation(profile, observed());
  assert.equal(checked.authenticated, false);
  assert.equal(checked.publicationComplete, false);
  assert.equal(checked.signerAuthority, false);
  assert.equal(checked.writerLeaseAuthority, false);
  assert.throws(() => joinProductionProducerEvidence([profile], [checked]));
});
refuse('PR producer refuses', input => {
  input.run.event = 'pull_request';
  input.attempt.event = 'pull_request';
});
refuse('non-default branch refuses', input => {
  input.run.head_branch = 'main';
  input.attempt.head_branch = 'main';
});
refuse('foreign head repository refuses', input => {
  input.run.head_repository.full_name = 'foreign/project';
});
refuse('stale attempt refuses', input => {
  input.attempt.run_attempt = 1;
});
refuse('foreign workflow refuses', input => {
  input.workflow.path = '.github/workflows/other.yml';
});
refuse('foreign source refuses', input => {
  input.job.head_sha = 'a'.repeat(40);
});
refuse('failed producer refuses', input => {
  input.job.conclusion = 'failure';
});
refuse('skipped unconfigured step refuses', input => {
  input.job.steps[0].conclusion = 'skipped';
});
refuse('skipped existing step needs independent artifact evidence', input => {
  input.job.steps[1].conclusion = 'skipped';
});
refuse('missing runtime step refuses', input => {
  input.job.steps.pop();
});
refuse('duplicate required step refuses', input => {
  input.job.steps.push({
    ...input.job.steps[0]
  });
});
refuse('incomplete step refuses', input => {
  input.job.steps[1].status = 'in_progress';
});
test('copied source profile refuses', () => assert.throws(() => validateProducerObservation({
  ...profile
}, observed())));
test('fake complete proof refuses', () => assert.throws(() => joinProductionProducerEvidence([profile], [{
  authenticated: true,
  publicationComplete: true,
  ...profile
}])));
test('unregistered source job refuses', () => assert.throws(() => productionProducerProfile(fixture, head, '.github/workflows/publication.yml', 'other')));
test('qualifier source drift refuses', () => {
  writeFileSync(join(fixture, 'scripts/lib/release-completion-producer.mjs'), 'changed source\n');
  git(['add', '.']);
  git(['commit', '-qm', 'source-only drift']);
  assert.throws(() => productionProducerProfile(fixture, git(['rev-parse', 'HEAD']), '.github/workflows/publication.yml', 'publish'));
});
test.after(() => rmSync(fixture, {
  recursive: true,
  force: true
}));
refuse('run protocol commit drift refuses', input => {
  input.run.head_sha = 'b'.repeat(40);
});
refuse('attempt repository drift refuses', input => {
  input.attempt.repository.full_name = 'other/project';
});
refuse('attempt event drift refuses', input => {
  input.attempt.event = 'workflow_dispatch';
});
refuse('unknown production event refuses', input => {
  input.run.event = 'schedule';
  input.attempt.event = 'schedule';
});
refuse('workflow identifier drift refuses', input => {
  input.workflow.id = 9;
});
refuse('producer run identifier drift refuses', input => {
  input.job.run_id = 9;
});
refuse('producer source name drift refuses', input => {
  input.job.name = 'Another publisher';
});
refuse('zero run identifier refuses', input => {
  input.run.id = 0;
  input.attempt.id = 0;
});
refuse('unfinished original run refuses', input => {
  input.run.status = 'in_progress';
});
test('caller delivery booleans are never accepted as Complete', () => {
  const input = observed();
  input.publicationComplete = true;
  input.writerLeaseAuthority = true;
  const checked = validateProducerObservation(profile, input);
  assert.equal(checked.publicationComplete, false);
  assert.equal(checked.writerLeaseAuthority, false);
  assert.throws(() => joinProductionProducerEvidence([{
    ...profile
  }], [{
    ...checked,
    authenticated: true
  }]));
});
function changedWorkflow(body) {
  writeFileSync(join(fixture, 'scripts/lib/release-completion-producer.mjs'), readFileSync(helper));
  writeFileSync(join(fixture, '.github/workflows/publication.yml'), body);
  git(['add', '.']);
  git(['commit', '-qm', 'source-only changed contract']);
  return git(['rev-parse', 'HEAD']);
}
test('matrix identity is refused until a qualified source expansion exists', () => {
  const revision = changedWorkflow('jobs:\n  publish:\n    strategy:\n      matrix:\n        target: [one, two]\n    steps:\n      - name: Publish\n        run: echo source-only\n');
  assert.throws(() => productionProducerProfile(fixture, revision, '.github/workflows/publication.yml', 'publish'), /matrix producer identity/);
});
test('dynamic producer names cannot mint a source profile', () => {
  const revision = changedWorkflow('jobs:\n  publish:\n    name: Publish ${{ matrix.target }}\n    steps:\n      - name: Publish\n        run: echo source-only\n');
  assert.throws(() => productionProducerProfile(fixture, revision, '.github/workflows/publication.yml', 'publish'), /dynamic producer name/);
});
test('unnamed run steps refuse incomplete source obligations', () => {
  const revision = changedWorkflow('jobs:\n  publish:\n    steps:\n      - run: echo source-only\n');
  assert.throws(() => productionProducerProfile(fixture, revision, '.github/workflows/publication.yml', 'publish'), /stable source name/);
});

import {productionMatrixProducerProfile, validateMatrixProducerObservation,
  joinProductionMatrixProducerEvidence} from '../../../scripts/lib/release-completion-producer.mjs';
let matrixFixtureSerial = 0;
function matrixFixture(body) {
  const revision = changedWorkflow(body + '\n# matrix fixture ' + ++matrixFixtureSerial + '\n');
  const matrix = productionMatrixProducerProfile(fixture, revision, '.github/workflows/publication.yml', 'publish');
  const evidence = observed();
  for (const record of [evidence.run, evidence.attempt]) record.head_sha = revision;
  evidence.jobs = matrix.jobNames.map((name, index) => ({id: 100 + index, run_id: 1, head_sha: revision,
    name, status: 'completed', conclusion: 'success', steps: [{name: 'Publish source', status: 'completed', conclusion: 'success'}]}));
  return {matrix, evidence};
}
const matrixWorkflow = 'jobs:\n  publish:\n    name: Target \${{ matrix.target }} \${{ matrix.flavor }}\n    strategy:\n      matrix:\n        target: [linux, windows]\n        flavor: [full, slim]\n        exclude:\n          - target: windows\n            flavor: slim\n        include:\n          - target: macos\n            flavor: full\n    steps:\n      - name: Publish source\n        run: echo source-only\n';
test('static axes exclude and include bind every required source job without delivery credit', () => {
  const {matrix, evidence} = matrixFixture(matrixWorkflow.replaceAll('\\$', '$'));
  assert.deepEqual(matrix.jobNames, ['Target linux full', 'Target linux slim', 'Target windows full', 'Target macos full']);
  const result = validateMatrixProducerObservation(matrix, evidence);
  assert.equal(result.matrixSize, 4); assert.equal(result.authenticated, false);
  assert.equal(result.publicationComplete, false); assert.equal(result.registryEvidence, 'Missing');
  assert.throws(() => joinProductionMatrixProducerEvidence(matrix, result));
});
test('matrix observations refuse missing duplicate failed skipped stale or foreign jobs', () => {
  const {matrix, evidence} = matrixFixture(matrixWorkflow.replaceAll('\\$', '$'));
  const changes = [value => value.jobs.pop(), value => value.jobs.push({...value.jobs[0], id: 999}),
    value => value.jobs[1].conclusion = 'failure', value => value.jobs[1].steps[0].conclusion = 'skipped',
    value => value.attempt.run_attempt++, value => value.jobs[1].head_sha = 'a'.repeat(40),
    value => value.run.repository.full_name = 'foreign/project'];
  for (const change of changes) { const value = structuredClone(evidence); change(value);
    assert.throws(() => validateMatrixProducerObservation(matrix, value)); }
  assert.throws(() => validateMatrixProducerObservation({...matrix}, evidence));
});
test('dynamic unknown and colliding source matrix identities retain refusal', () => {
  for (const body of [matrixWorkflow.replace('target: [linux, windows]', 'target: \${{ fromJSON(needs.source.outputs.targets) }}'),
    matrixWorkflow.replace('Target \${{ matrix.target }} \${{ matrix.flavor }}', 'Target \${{ env.TARGET }}'),
    matrixWorkflow.replace('Target \${{ matrix.target }} \${{ matrix.flavor }}', 'Constant name'),
    matrixWorkflow.replace('target: [linux, windows]', 'target: [linux, linux]'),
    matrixWorkflow.replace('    strategy:', '    if: true\n    strategy:')]) {
    const revision = changedWorkflow(body.replaceAll('\\$', '$'));
    assert.throws(() => productionMatrixProducerProfile(fixture, revision, '.github/workflows/publication.yml', 'publish'));
  }
});
test('include overwrite rules and include-only jobs retain source order', () => {
  const body = 'jobs:\n  publish:\n    name: \${{ matrix.fruit }} \${{ matrix.animal }} \${{ matrix.color }}\n    strategy:\n      matrix:\n        fruit: [apple, pear]\n        animal: [cat, dog]\n        include:\n          - color: green\n          - color: pink\n            animal: cat\n    steps:\n      - name: Publish source\n        run: echo source-only\n';
  const {matrix} = matrixFixture(body.replaceAll('\\$', '$'));
  assert.deepEqual(matrix.jobNames, ['apple cat pink', 'apple dog green', 'pear cat pink', 'pear dog green']);
});

test('static source names refuse another job or reusable descendant ambiguity', () => {
  for (const extra of ['  other:\n    name: Target linux full\n    steps:\n      - run: echo fixture\n',
    '  other:\n    uses: ./.github/workflows/child.yml\n',
    '  other:\n    name: \${{ needs.source.outputs.name }}\n    steps:\n      - run: echo fixture\n']) {
    const revision = changedWorkflow((matrixWorkflow + extra).replaceAll('\\$', '$'));
    assert.throws(() => productionMatrixProducerProfile(fixture, revision, '.github/workflows/publication.yml', 'publish'));
  }
});

test('foreign unnamed matrix display names refuse source registration', () => {
  const extra = '  other:\n    strategy:\n      matrix:\n        target: [linux]\n    steps:\n      - name: Foreign source\n        run: echo source-only\n';
  const revision = changedWorkflow((matrixWorkflow + extra).replaceAll('\\$', '$'));
  assert.throws(() => productionMatrixProducerProfile(fixture, revision,
    '.github/workflows/publication.yml', 'publish'), /foreign generated matrix display-name namespace/);
});
