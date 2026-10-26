import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';

const root = path.resolve(import.meta.dirname, '../../..');
const harness = 'experiments/agentic_cli_matrix';
const workflow = YAML.parse(fs.readFileSync(path.join(root, '.github/workflows/agentic-cli-matrix.yml'), 'utf8'));
const failure = JSON.parse(fs.readFileSync(new URL('../fixtures/ci/recorded-replay-checkout-timeout.json', import.meta.url)));

test('recorded replay reserves execution time within its original job budget and checks out only its source closure', () => {
  const job = workflow.jobs.replay;
  assert.equal(job['timeout-minutes'] * 60, failure.jobBudgetSeconds);
  const fetchSeconds = (Date.parse(failure.fetchCompleted) - Date.parse(failure.fetchStarted)) / 1000;
  assert(fetchSeconds > failure.jobBudgetSeconds * 0.9);
  assert.equal(failure.replayStarted, false);
  const checkout = job.steps.find(step => step.uses?.startsWith('actions/checkout@'));
  assert.equal(checkout.with['sparse-checkout'], harness);
  assert.equal(checkout.with['persist-credentials'], false);
  assert(checkout['timeout-minutes'] > 0 && checkout['timeout-minutes'] < job['timeout-minutes']);
  assert(Number(job.env.GIT_HTTP_LOW_SPEED_LIMIT) > 0);
  assert(Number(job.env.GIT_HTTP_LOW_SPEED_TIME) > 0
    && Number(job.env.GIT_HTTP_LOW_SPEED_TIME) < checkout['timeout-minutes'] * 60);
  assert.equal(job.steps.find(step => step.name === 'Replay every committed transcript').run, harness + '/replay.sh');
});

function command(name) {
  for (const directory of process.env.PATH.split(path.delimiter)) {
    const candidate = path.join(directory, name);
    try { fs.accessSync(candidate, fs.constants.X_OK); return fs.realpathSync(candidate); } catch {}
  }
  assert.fail('Required offline fixture utility is missing: ' + name);
}

test('every original transcript replays with only the sparse harness, jq and shell utilities', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-sparse-recorded-replay-'));
  try {
    const target = path.join(directory, harness), bin = path.join(directory, 'bin');
    fs.mkdirSync(target, {recursive:true}); fs.mkdirSync(bin);
    for (const name of ['replay.sh', 'lib.sh']) fs.copyFileSync(path.join(root,harness,name), path.join(target,name));
    const records = path.join(root,harness,'recorded'), clients = [], cases = [];
    let total = 0;
    for (const client of fs.readdirSync(records, {withFileTypes:true})) {
      assert(client.isDirectory() && /^[a-z0-9-]+$/u.test(client.name));
      clients.push(client.name); const destination = path.join(target,'recorded',client.name);
      fs.mkdirSync(destination,{recursive:true});
      for (const file of fs.readdirSync(path.join(records,client.name),{withFileTypes:true})) {
        assert(file.isFile() && file.name.endsWith('.jsonl'));
        const source = path.join(records,client.name,file.name);
        total += fs.statSync(source).size; assert(total <= 8388608 && cases.length < 256);
        fs.copyFileSync(source,path.join(destination,file.name));
        cases.push(client.name+'/'+file.name.slice(0,-6));
      }
    }
    assert(cases.length > 0);
    // No compiler, product binary, Node, curl or GitHub CLI is on this PATH.
    for (const name of ['bash','jq','dirname','basename','wc','tr','head','ls']) fs.symlinkSync(command(name),path.join(bin,name));
    const observed = spawnSync(path.join(bin,'bash'),[path.join(target,'replay.sh'),...clients],{
      cwd:directory,encoding:'utf8',timeout:15000,maxBuffer:65536,
      env:{PATH:bin,BASH_ENV:'/dev/null',MODEL:'formal-ai',MATRIX_RESOLVED_DIR:path.join(directory,'unused-resolved')},
    });
    assert.equal(observed.error,undefined);assert.equal(observed.signal,null);
    assert.equal(observed.status,0,observed.stderr+'\n'+observed.stdout);
    for (const name of cases) assert(observed.stdout.includes('ok '+name+':'), 'original transcript not replayed: '+name);
    assert(observed.stdout.includes('== replay OK =='));
    assert(!fs.existsSync(path.join(directory,'rust')));
  } finally { fs.rmSync(directory,{recursive:true,force:true}); }
});
