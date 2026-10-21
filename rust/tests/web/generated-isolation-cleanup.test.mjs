import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const workflow = fs.readFileSync(new URL('../../../.github/workflows/e2e-isolation.yml', import.meta.url), 'utf8');
const cleanup = workflow.slice(workflow.lastIndexOf('      - name: Cleanup'));
const body = cleanup.slice(cleanup.indexOf('        run: |\n') + '        run: |\n'.length)
  .split('\n').map(line => line.startsWith('          ') ? line.slice(10) : line).join('\n');
function execute({partial = false, adopted = false, privateRepository = false, changedRepository = false, unsafePurpose = false} = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'generated-isolation-proof-'));
  try {
    const purpose = unsafePurpose ? 'owned#check' : 'owned-check', run = '20261021000000-1234';
    const branch = `e2e/${purpose}/${run}/run`, repository = 'example/project';
    const target = privateRepository ? `example/formal-ai-e2e-${run}` : repository;
    const issue = {title: `e2e: ${purpose} (1234)`, state: 'open', labels: [{name: 'e2e-task'}],
      body: `Isolated run of **${purpose}**.\n\n- orphan branch: \`${branch}\`\n`
        + `- dispatched by: https://github.com/${repository}/actions/runs/1234\n- credential layer: default\n\n`
        + 'The cleanup job closes this issue and deletes the branch at the end of the run.'};
    if (adopted) issue.body += '\nHuman feature requirements added after the test.';
    const specification = {issue, repository: {full_name: target, private: !changedRepository,
      description: changedRepository ? 'Adopted user repository' : `Isolated e2e run ${purpose} from ${repository} (1234)`}};
    const fixture = path.join(directory, 'fixture.json'), capture = path.join(directory, 'calls.jsonl');
    fs.writeFileSync(fixture, JSON.stringify(specification));
    fs.writeFileSync(path.join(directory, 'gh'), '#!' + process.execPath + '\n'
      + `const fs=require('node:fs');const args=process.argv.slice(2);`
      + String.raw`fs.appendFileSync(process.env.ISOLATION_CAPTURE,JSON.stringify(args)+'\n');`
      + `const fixture=JSON.parse(fs.readFileSync(process.env.ISOLATION_FIXTURE));`
      + `if(args[0]==='api'&&args[1]!=='-X')console.log(JSON.stringify(args[1].includes('/issues/')?fixture.issue:fixture.repository));`,
    {mode: 0o700});
    const result = spawnSync('bash', ['-c', body], {encoding: 'utf8', timeout: 10000, maxBuffer: 65536,
      env: {...process.env, PATH: directory + path.delimiter + process.env.PATH,
        ISOLATION_FIXTURE: fixture, ISOLATION_CAPTURE: capture,
        GITHUB_RUN_ID: '1234', GITHUB_REPOSITORY: repository, GITHUB_REPOSITORY_OWNER: 'example',
        GITHUB_SERVER_URL: 'https://github.com', PURPOSE: purpose, CREDENTIAL_LAYER: 'default',
        OWNED_RUN_ID: partial ? '' : run, BRANCH: partial ? '' : branch,
        ISSUE: partial ? '' : '42', TARGET_REPO: partial ? '' : target}});
    assert.equal(result.status, 0, result.stderr); assert.equal(result.error, undefined);
    const calls = fs.existsSync(capture) ? fs.readFileSync(capture, 'utf8').trim().split('\n').filter(Boolean).map(JSON.parse) : [];
    return {calls, stdout: result.stdout, branch};
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
}
test('missing outputs and unsafe ref components cause no resource effects', () => {
  for (const options of [{partial: true}, {unsafePurpose: true}]) assert.deepEqual(execute(options).calls, []);
});
test('an adopted feature issue is retained despite matching generated title', () => {
  const {calls} = execute({adopted: true});
  assert.equal(calls.some(args => args[0] === 'issue' && args[1] === 'close'), false);
});
test('a changed private test repository is never deleted', () => {
  const {calls} = execute({privateRepository: true, changedRepository: true});
  assert.equal(calls.some(args => args[0] === 'repo' && args[1] === 'delete'), false);
  assert.equal(calls.some(args => args.includes('DELETE')), false);
});
test('exact current-run issue and private repository proofs authorize only owned cleanup', () => {
  const normal = execute();
  assert.equal(normal.calls.filter(args => args[0] === 'issue' && args[1] === 'close').length, 1);
  const deleted = normal.calls.filter(args => args.includes('DELETE')).map(args => args.at(-1));
  assert.deepEqual(deleted, [`repos/example/project/git/refs/heads/${normal.branch}`,
    `repos/example/project/git/refs/heads/${normal.branch}-run`]);
  const separate = execute({privateRepository: true});
  assert.equal(separate.calls.filter(args => args[0] === 'repo' && args[1] === 'delete').length, 1);
  assert.equal(separate.calls.some(args => args[0] === 'issue'), false);
});
