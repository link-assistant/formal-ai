import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import test from 'node:test';
import YAML from 'yaml';
import {dependencyPublication, prepareDependencyReview} from '../../../scripts/dependency-update-publication.mjs';

test('default credential may publish ordinary updates but preserves whole workflow-changing updates', () => {
  assert.equal(dependencyPublication(['package.json', 'bun.lock'], 'default').publish, true);
  const paths = ['package.json', '.github/workflows/desktop-release.yml'];
  const decision = dependencyPublication(paths, 'default');
  assert.equal(decision.publish, false);
  assert.equal(decision.reason, 'default-token-cannot-write-workflows');
  assert.equal(decision.publicationApplied, false);
  assert.deepEqual(decision.workflowPaths, paths.slice(1));
  for (const layer of ['app', 'token']) {
    assert.equal(dependencyPublication(paths, layer).publish, true);
    assert.equal(dependencyPublication(paths, layer).publicationApplied, false);
  }
  assert.throws(() => dependencyPublication(paths, 'unknown'));
  assert.throws(() => dependencyPublication(Array(10001).fill('package.json'), 'default'));
});

test('real Git patch retains all updates without publishing or rewriting the worktree', () => {
  const root = fs.mkdtempSync(path.join(tmpdir(), 'formal-ai-dependency-review-'));
  const cwd = path.join(root, 'repository'), hooks = path.join(root, 'empty-hooks');
  fs.mkdirSync(path.join(cwd, '.github/workflows'), {recursive: true}); fs.mkdirSync(hooks);
  const git = args => execFileSync('git', ['-c', 'core.hooksPath=' + hooks, ...args], {
    cwd, encoding: 'utf8', timeout: 5000,
    env: {...process.env, DOCKER_NO_PRUNE: '1', CARGO_TEST_NO_PRUNE: '1'},
  });
  try {
    git(['init', '-q', '-b', 'main']);
    fs.writeFileSync(path.join(cwd, 'package.json'), '{"name":"fixture","version":"1.0.0"}\n');
    fs.writeFileSync(path.join(cwd, '.github/workflows/dependency.yml'), 'name: dependency\n# version one\n');
    git(['add', '.']);
    git(['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'fixture']);
    fs.writeFileSync(path.join(cwd, 'package.json'), '{"name":"fixture","version":"2.0.0"}\n');
    fs.writeFileSync(path.join(cwd, '.github/workflows/dependency.yml'), 'name: dependency\n# version two\n');
    const before = git(['diff', '--binary', '--no-renames', 'HEAD', '--']);
    const report = prepareDependencyReview({cwd, layer: 'default', directory: path.join(root, 'review')});
    const patch = fs.readFileSync(path.join(root, 'review/update.patch'));
    assert.equal(report.publish, false); assert.equal(report.publicationApplied, false);
    assert.match(patch.toString(), /version two/); assert.match(patch.toString(), /2\.0\.0/);
    assert.equal(report.patchSha256, createHash('sha256').update(patch).digest('hex'));
    assert.equal(report.patchBytes, patch.length);
    assert.equal(git(['diff', '--binary', '--no-renames', 'HEAD', '--']), before);
    assert.deepEqual(JSON.parse(fs.readFileSync(path.join(root, 'review/review.json'))), report);
    const remote = path.join(root, 'remote.git'), branch = 'dependencies/latest/123';
    git(['init', '--bare', '-q', '-b', 'main', remote]); git(['remote', 'add', 'origin', remote]);
    const candidate = prepareDependencyReview({cwd, layer: 'app', branch, directory: path.join(root, 'candidate')});
    assert.equal(candidate.publish, true); assert.equal(candidate.branch, branch);
    git(['push', '-q', 'origin', 'HEAD:refs/heads/' + branch]);
    const existing = git(['ls-remote', 'origin', 'refs/heads/' + branch]);
    const retry = prepareDependencyReview({cwd, layer: 'app', branch, directory: path.join(root, 'retry')});
    assert.equal(retry.publish, false); assert.equal(retry.reason, 'immutable-update-branch-already-exists');
    assert.match(retry.existingCommit, /^[a-f0-9]{40}$/u);
    assert.equal(git(['ls-remote', 'origin', 'refs/heads/' + branch]), existing);
    assert.match(fs.readFileSync(path.join(root, 'retry/update.patch'), 'utf8'), /version two/);
    git(['mv', '.github/workflows/dependency.yml', 'moved.yml']);
    const renamed = prepareDependencyReview({cwd, layer: 'default', directory: path.join(root, 'renamed')});
    assert.equal(renamed.publish, false, 'moving a workflow out still requires workflow permission');
    assert.ok(renamed.workflowPaths.includes('.github/workflows/dependency.yml'));
    git(['remote', 'remove', 'origin']);
    assert.throws(() => prepareDependencyReview({cwd, layer: 'app', branch: 'dependencies/latest/124',
      directory: path.join(root, 'unverifiable')}), /bounded dependency review command failed/);
    assert.equal(fs.existsSync(path.join(root, 'unverifiable')), false);
  } finally {fs.rmSync(root, {recursive: true, force: true});}
});

test('dependency workflow requests the missing App scope and safely gates publication', () => {
  const workflow = YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/dependencies-latest.yml', import.meta.url), 'utf8'));
  const job = workflow.jobs.update, steps = job.steps;
  const credentials = steps.find(step => step.uses === './.github/actions/automation-token');
  assert.equal(credentials.with['permission-workflows'], 'write');
  assert.ok(steps.indexOf(credentials) > steps.findIndex(step => step.name === 'Run registered gates (stage web)'),
    'short-lived write credentials are minted after long validation');
  assert.match(credentials.if, /create-pull-request == true/);
  assert.equal(steps.find(step => step.name === 'Refresh the registry snapshot the gate reads').env.GH_TOKEN,
    '${{ github.token }}');
  assert.equal(job.permissions.actions, 'write');
  const review = steps.find(step => step.id === 'publication');
  assert.equal(review.run, 'node scripts/dependency-update-publication.mjs');
  assert.equal(review.env.AUTOMATION_LAYER, "${{ steps.automation.outputs.layer || 'default' }}");
  assert.ok(steps.findIndex(step => step.name === 'Run registered gates (stage web)') < steps.indexOf(review));
  const upload = steps.find(step => step.with?.name === 'dependency-update-review');
  assert.equal(upload.uses, 'actions/upload-artifact@v7');
  const publication = steps.find(step => step.id === 'pull-request');
  assert.match(publication.if, /steps\.publication\.outputs\.publish == 'true'/);
  assert.equal(publication.with.branch, '${{ steps.publication.outputs.branch }}');
  assert.equal(publication.with['delete-branch'], false);
  assert.ok(steps.indexOf(upload) < steps.indexOf(publication));
  const dispatch = steps.find(step => step.uses === './.github/actions/dispatch-checks');
  assert.match(dispatch.if, /layer == 'default'/); assert.match(dispatch.if, /operation == 'created'/);
  assert.match(dispatch.if, /operation == 'updated'/);
});
