// Model Actions' process boundary; exercise the real strict authority validator without network/native tools.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import { execFileSync, spawnSync } from 'node:child_process';
const root = fileURLToPath(new URL('../../../', import.meta.url));
const YAML = createRequire(path.join(root, 'package.json'))('yaml');
const workflow = YAML.parse(fs.readFileSync(path.join(root, '.github/workflows/container-images.yml'), 'utf8'));
const importName = 'Import and independently authorize the selected source';
const owners = Object.entries(workflow.jobs).filter(([, job]) => job.steps?.some(step => step.name === importName));
assert.equal(owners.length, 5);
function fixture(t) {
  const cwd = fs.mkdtempSync(path.join(tmpdir(), 'authority-boundary-'));
  t.after(() => fs.rmSync(cwd, {
    recursive: true,
    force: true
  }));
  const git = (...args) => execFileSync('git', args, {
    cwd,
    encoding: 'utf8',
    stdio: 'pipe'
  }).trim();
  git('init', '--initial-branch=main');
  git('config', 'user.name', 'Fixture');
  git('config', 'user.email', 'fixture@example.invalid');
  fs.writeFileSync(path.join(cwd, 'source.txt'), 'real source\n');
  git('add', '.');
  git('commit', '-m', 'source');
  const head = git('rev-parse', 'HEAD');
  git('tag', 'v1.2.3');
  const protocol = path.join(cwd, 'protocol');
  fs.mkdirSync(protocol);
  fs.mkdirSync(path.join(cwd, 'container-source'));
  const actual = JSON.stringify(new URL('../../../scripts/native-release-trust.mjs', import.meta.url).href);
  fs.writeFileSync(path.join(protocol, 'native-release-trust.mjs'), `import fs from 'node:fs';export {validateSelectedSourceAuthority} from ${actual};if(process.argv[1].endsWith('native-release-trust.mjs')){if(process.env.FIXTURE_DENY==='true')throw Error('independent authorization refused');fs.appendFileSync(process.env.GITHUB_ENV,'NATIVE_AUTHORIZED_RELEASE_COMMIT='+process.env.FIXTURE_HEAD+'\\nNATIVE_AUTHORIZED_DEFAULT_COMMIT='+process.env.FIXTURE_HEAD+'\\n');}`);
  fs.writeFileSync(path.join(protocol, 'native-release-source.mjs'), `import fs from 'node:fs';fs.writeFileSync('import-observed.json',JSON.stringify({event:process.env.NATIVE_RELEASE_EVENT}));`);
  fs.writeFileSync(path.join(cwd, 'container-source/source-selection.json'), JSON.stringify({
    release_tag: 'v1.2.3',
    package_version: '1.2.3',
    source_commit: head,
    selected_head: head,
    bundle: null
  }));
  const environment = {
    ...process.env,
    CONTAINER_PUBLISH: 'true',
    NATIVE_RELEASE_TAG: 'v1.2.3',
    NATIVE_RELEASE_EVENT: 'workflow_call',
    FIXTURE_HEAD: head,
    FORMAL_AI_CONTAINER_PROTOCOL_DIR: protocol,
    GITHUB_ENV: path.join(cwd, 'github-env')
  };
  delete environment.NATIVE_AUTHORIZED_RELEASE_COMMIT;
  delete environment.NATIVE_AUTHORIZED_DEFAULT_COMMIT;
  return {
    cwd,
    environment
  };
}
function shell(script, f) {
  return spawnSync('/bin/bash', ['-e', '-o', 'pipefail', '-c', script], {
    cwd: f.cwd,
    env: f.environment,
    encoding: 'utf8',
    timeout: 10000
  });
}
function nextStep(f) {
  const text = fs.readFileSync(f.environment.GITHUB_ENV, 'utf8');
  const expected = ['NATIVE_AUTHORIZED_RELEASE_COMMIT', 'NATIVE_AUTHORIZED_DEFAULT_COMMIT'];
  const lines = text.trimEnd().split('\n');
  assert.equal(lines.length, 2);
  for (const line of lines) {
    const [key, value] = line.split('=');
    assert(expected.includes(key));
    assert.match(value, /^[a-f0-9]{40}$/u);
    f.environment[key] = value;
  }
}
for (const [owner, job] of owners) test(owner + ' consumes independent authority only after an Actions step boundary', t => {
  const f = fixture(t),
    index = job.steps.findIndex(step => step.name === importName),
    step = job.steps[index],
    authorize = job.steps[index - 1];
  // Original workflow reproducibly loses the child-written environment in this same shell.
  if (step.run.includes('node "$FORMAL_AI_CONTAINER_PROTOCOL_DIR/native-release-trust.mjs"')) {
    const old = shell(step.run, f);
    assert.notEqual(old.status, 0);
    assert.match(old.stderr, /undefined/);
    assert.fail('authority producer and consumer share a run step');
  }
  assert.equal(authorize.name, 'Independently authorize the published release source');
  assert.equal(authorize.if, "env.CONTAINER_PUBLISH == 'true'");
  assert.equal(authorize.env.GH_TOKEN, '${{ github.token }}');
  const first = shell(authorize.run, f);
  assert.equal(first.status, 0, first.stderr);
  assert.equal(f.environment.NATIVE_AUTHORIZED_RELEASE_COMMIT, undefined, 'child cannot modify parent env');
  nextStep(f);
  const imported = shell(step.run, f);
  assert.equal(imported.status, 0, imported.stderr);
  assert.equal(JSON.parse(fs.readFileSync(path.join(f.cwd, 'import-observed.json'))).event, 'workflow_dispatch');
  // The real unchanged validator must still reject the wrong independently supplied commit.
  f.environment.NATIVE_AUTHORIZED_RELEASE_COMMIT = 'f'.repeat(40);
  assert.notEqual(shell(step.run, f).status, 0);
});
test('independent authorization failure stops before import', t => {
  const f = fixture(t);
  f.environment.FIXTURE_DENY = 'true';
  const [, job] = owners[0],
    index = job.steps.findIndex(s => s.name === importName);
  assert.notEqual(shell(job.steps[index - 1].run, f).status, 0);
  assert(!fs.existsSync(path.join(f.cwd, 'import-observed.json')));
});
test('unprivileged validation never uses published authority', t => {
  const f = fixture(t);
  f.environment.CONTAINER_PUBLISH = 'false';
  const record = path.join(f.cwd, 'container-source/source-selection.json');
  fs.writeFileSync(record, JSON.stringify({
    release_tag: null
  }));
  const [, job] = owners[0],
    step = job.steps.find(s => s.name === importName);
  const result = shell(step.run, f);
  assert.equal(result.status, 0, result.stderr);
  assert(!fs.existsSync(f.environment.GITHUB_ENV));
  fs.writeFileSync(record, JSON.stringify({
    release_tag: 'v1.2.3'
  }));
  assert.notEqual(shell(step.run, f).status, 0);
});
