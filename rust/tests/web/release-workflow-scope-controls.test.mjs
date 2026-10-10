import {runStagedReleaseGenerator} from '../../../scripts/generate-staged-release.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';
const root = fileURLToPath(new URL('../../../', import.meta.url));
const writers=['git push','scripts/push-to-shared-branch.sh','scripts/version-and-commit.rs'];
function proveGitWriterCredential(job){
 assert.ok(job&&Array.isArray(job.steps),'declared ordered steps required');
 assert.equal(job.defaults?.run?.['working-directory'],undefined,'unknown job working directory');
 let rootCheckout=null;let pushes=0;
 for(const step of job.steps){
  if(step.uses?.startsWith('actions/checkout@')){
   const settings=step.with??{};const path=settings.path??'.';
   assert.equal(typeof path,'string');assert.ok(!path.includes('${{')&&!path.includes('..')&&!path.startsWith('/'),'checkout path unknown');
   if (settings['persist-credentials'] !== undefined) assert.equal(typeof settings['persist-credentials'], 'boolean', 'explicit persistence must be typed');
   if(path==='.'||path==='')rootCheckout={persist:settings['persist-credentials']!==false,token:settings.token??'${{ github.token }}'};
  }
  const code=(step.run??'').split('\n').filter(line=>!line.trimStart().startsWith('#')).join('\n');
  if(!writers.some(op=>code.includes(op))&&!step.uses?.startsWith('peter-evans/create-pull-request'))continue;
  assert.equal(step['working-directory'],undefined,'push working directory not proven');
  assert.ok(!/(?:^|[;&\n])\s*(?:cd|git\s+-C)\s/u.test(code),'push changes repository scope');
  assert.ok(rootCheckout?.persist,'preceding root checkout must retain push credential');
  assert.ok(['${{ github.token }}','${{ secrets.GITHUB_TOKEN }}'].includes(rootCheckout.token),'unknown root token authority');
  pushes++;
 }
 assert.ok(pushes>0,'actual git writer required');return pushes;
}

test('all original writers and checked dormant source stages retain only scoped push credentials', () => {
 const roles = [['dependencies-latest.yml', 'update'], ['e2e-isolation.yml', 'isolate'],
  ['external-benchmarks.yml', 'external-benchmarks'], ['release.yml', 'auto-release'],
  ['release.yml', 'manual-release'], ['release.yml', 'changelog-pr'],
  ['release-staged.yml', 'auto_prepare-source'], ['release-staged.yml', 'manual_prepare-source']];
 for (const [file, role] of roles) {
  const workflow = YAML.parse(fs.readFileSync(join(root, '.github/workflows', file), 'utf8'));
  assert.ok(proveGitWriterCredential(workflow.jobs[role]) > 0);
 }
 const retained = {uses: 'actions/checkout@v7', with: {token: '${{ secrets.GITHUB_TOKEN }}'}};
 const push = {run: 'git push origin HEAD:main'};
 const isolated = {uses: 'actions/checkout@v7', with: {path: '_protocol', 'persist-credentials': false}};
 assert.equal(proveGitWriterCredential({steps: [retained, push, isolated]}), 1);
 for (const steps of [[push, retained], [{...retained, with: {'persist-credentials': false}}, push],
  [isolated, push], [{...retained, with: {'persist-credentials': 'false'}}, push], [{...retained, with: {token: '${{ inputs.token }}'}}, push],
  [retained, {run: 'cd _protocol\ngit push'}], [retained, {...push, 'working-directory': '_protocol'}]]) {
  assert.throws(() => proveGitWriterCredential({steps}));
 }
});
test('budget wrapper preserves stdout/status and terminates the complete harmless process group', async () => {
 const directory = fs.mkdtempSync(join(fs.realpathSync(tmpdir()), 'release-deadline-controls-'));
 const sentinel = join(directory, 'escaped');
 const environment = {...process.env, TEST_BUDGET_ENFORCE: 'true', TEST_BUDGET_GRACE_SECONDS: '0',
  TEST_BUDGET_POLL_SECONDS: '1', TEST_WARN_RATIO_PERCENT: '70', RUSTC_WRAPPER: ''};
 const childCode = "import fs from 'node:fs';setTimeout(()=>fs.writeFileSync(" + JSON.stringify(sentinel) + ",'escaped'),3500)";
 const cases = [['success', 'console.log("preserved stdout")', 0],
  ['failure', 'console.error("preserved failure");process.exit(7)', 7],
  ['timeout', "import {spawn} from 'node:child_process';spawn(process.execPath,['--input-type=module','-e'," +
   JSON.stringify(childCode) + "],{stdio:'inherit'});setInterval(()=>{},1000)", 124]];
 try {
  for (const [label, code, expected] of cases) {
   const result = spawnSync('bash', [join(root, 'scripts/run-with-budget-warning.sh'), label === 'timeout' ? '1' : '10',
    label, process.execPath, '--input-type=module', '-e', code], {env: environment, encoding: 'utf8', timeout: 10000});
   assert.equal(result.status, expected, result.stderr);
   if (label === 'success') assert.match(result.stdout, /preserved stdout/u);
   if (label === 'failure') assert.match(result.stderr, /preserved failure/u);
   if (label === 'timeout') assert.match(result.stderr, /exceeded its execution budget/u);
  }
  await new Promise(resolve => setTimeout(resolve, 3500));
  assert.equal(fs.existsSync(sentinel), false, 'expired command descendants must be terminated');
 } finally {fs.rmSync(directory, {recursive: true, force: true});}
});

test('maintained generator checks all five exact source-bound derivative outputs', () => {
 const directory = fs.mkdtempSync(join(fs.realpathSync(tmpdir()), 'staged-generator-controls-'));
 try {
  runStagedReleaseGenerator(['--check']);
  runStagedReleaseGenerator(['--write', '--directory', directory]);
  runStagedReleaseGenerator(['--check', '--directory', directory]);
  for (const name of ['release-staged.yml', 'candidate-release-caller.yml', 'candidate-output-bindings.json',
   'original-release-workflow.yml', 'stage-source-coverage.json']) {
   const path = join(directory, name), original = fs.readFileSync(path);
   fs.writeFileSync(path, Buffer.concat([original, Buffer.from('\nspoof')]));
   assert.throws(() => runStagedReleaseGenerator(['--check', '--directory', directory]));
   fs.writeFileSync(path, original);
  }
  assert.throws(() => runStagedReleaseGenerator(['--write']));
  assert.throws(() => runStagedReleaseGenerator(['--check', '--unknown']));
 } finally {fs.rmSync(directory, {recursive: true, force: true});}
});
