import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import vm from 'node:vm';
import {createRequire} from 'node:module';
import test from 'node:test';

const YAML = createRequire(import.meta.url)('yaml');
const read = file => readFileSync(new URL('../../' + file, import.meta.url), 'utf8');
const release = YAML.parse(read('.github/workflows/release.yml'));
const pages = YAML.parse(read('.github/workflows/pages-artifact.yml'));
const source = 'a'.repeat(40), child = 'b'.repeat(40);
function expression(text, context) {
  const code = text.trim().replace(/^\$\{\{\s*|\s*\}\}$/gu, '')
    .replace(/needs\.([\w-]+)/gu, (_, name) => 'needs[' + JSON.stringify(name) + ']');
  return vm.runInNewContext(code, context, {timeout: 1000});
}
function context(mode, status, options = {}) {
  return {github: {sha: source, ref: options.ref ?? 'refs/heads/main',
    event_name: mode === 'auto' ? 'push' : 'workflow_dispatch',
    event: {inputs: {release_mode: options.releaseMode ?? 'instant'}}},
    needs: {build: {result: options.build ?? 'success'},
      'auto-release': {result: mode === 'auto' ? status : 'skipped', outputs: {}},
      'manual-release': {result: mode === 'manual' ? status : 'skipped', outputs: {}}},
    cancelled: () => options.cancelled ?? false};
}

for (const mode of ['auto', 'manual']) {
  for (const status of ['failure', 'skipped']) {
    test('validated main Pages still runs after ' + mode + ' publication is ' + status, () => {
      const c = context(mode, status), job = release.jobs['build-pages'];
      assert.equal(expression(job.if, c), true);
      assert.equal(expression(job.with.sha, c), source);
    });
  }
  test('successful ' + mode + ' publication retains the resolved release child', () => {
    const c = context(mode, 'success');
    c.needs[mode + '-release'].outputs.pages_sha = child;
    assert.equal(expression(release.jobs['build-pages'].if, c), true);
    assert.equal(expression(release.jobs['build-pages'].with.sha, c), child);
  });
}
for (const options of [{build: 'failure'}, {build: 'cancelled'}, {cancelled: true},
  {ref: 'refs/heads/feature'}]) {
  test('Pages refuses the invalid build context ' + JSON.stringify(options), () => {
    assert.equal(expression(release.jobs['build-pages'].if, context('auto', 'failure', options)), false);
  });
}
test('a manual checks-only run cannot publish Pages', () => {
  assert.equal(expression(release.jobs['build-pages'].if,
    context('manual', 'failure', {releaseMode: 'checks'})), false);
});

test('checkout selects the immutable triggering commit on fallback and only a trusted branch for a release child', () => {
  const checkouts = pages.jobs.build.steps.filter(step => step.uses === 'actions/checkout@v7');
  for (const requested of [source, child]) {
    const c = {github: {sha: source, ref: 'refs/heads/main'}, inputs: {sha: requested}};
    const selected = checkouts.filter(step => step.if === undefined || expression(step.if, c));
    assert.equal(selected.length, 1);
    assert.equal(expression(selected[0].with.ref, c), requested === source ? source : c.github.ref);
    assert.equal(selected[0].with['persist-credentials'], false);
    assert.doesNotMatch(selected[0].with.ref, /inputs\.|steps\./u);
  }
});

test('the real Git identity check rejects a foreign child and accepts a trusted checkout', t => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-pages-source-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const env = {...process.env, GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_NOSYSTEM: '1',
    GIT_AUTHOR_NAME: 'Pages fixture', GIT_AUTHOR_EMAIL: 'pages@example.invalid',
    GIT_COMMITTER_NAME: 'Pages fixture', GIT_COMMITTER_EMAIL: 'pages@example.invalid'};
  const git = args => {
    const result = spawnSync('git', args, {cwd: directory, env, encoding: 'utf8'});
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  git(['init', '--initial-branch=main']);
  writeFileSync(join(directory, 'source.txt'), 'triggering source\n');
  git(['add', 'source.txt']); git(['commit', '-m', 'triggering source']);
  const triggering = git(['rev-parse', 'HEAD']);
  writeFileSync(join(directory, 'source.txt'), 'release child\n');
  git(['commit', '-am', 'release child']);
  const releaseChild = git(['rev-parse', 'HEAD']);
  const verify = pages.jobs.build.steps.find(step =>
    step.name === 'Verify trusted release branch matches the requested artifact commit').run;
  function check(requested) {
    return spawnSync('bash', ['-c', verify], {cwd: directory,
      env: {...env, PAGES_DEPLOY_SHA: requested}, encoding: 'utf8'});
  }
  assert.equal(check(triggering).status, 1);
  assert.equal(check(releaseChild).status, 0);
  git(['checkout', '--detach', triggering]);
  assert.equal(check(triggering).status, 0);
  assert.equal(check(releaseChild).status, 1);
});
