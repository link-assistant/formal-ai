// Fixed CI job clock and process-group deadline. No caller duration override.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
export const ENGINE_JOB_MILLISECONDS=1200000;
export const ENGINE_COLD_SECONDS=475;
export function remainingSeconds(clock,environment,now=Date.now()) {
  assert.equal(clock.schema,'browser-engine-job-clock/v1');
  assert.equal(environment.CI,'true');assert.equal(environment.GITHUB_JOB,'package');
  for(const field of ['GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT']) {
    assert.match(environment[field]??'',/^[1-9][0-9]*$/u);
    assert.equal(clock[field],environment[field]);
  }
  assert.equal(clock.GITHUB_WORKFLOW_SHA,environment.GITHUB_WORKFLOW_SHA);
  assert.match(clock.GITHUB_WORKFLOW_SHA,/^[a-f0-9]{40}$/u);
  assert(Number.isSafeInteger(clock.started));assert(clock.started<=now);
  const seconds=Math.floor((clock.started+ENGINE_JOB_MILLISECONDS-now)/1000)-5;
  assert(seconds>0,'absolute package job deadline exhausted');return seconds;
}
export function deadlineArguments(seconds,argumentsList) {
  assert(Number.isInteger(seconds)&&seconds>0&&seconds<=1195);
  assert(Array.isArray(argumentsList)&&argumentsList.length>0);
  assert(argumentsList.every(value=>typeof value==='string'&&!value.includes('\0')));
  return ['--signal=TERM','--kill-after=5s',seconds+'s',...argumentsList];
}
export function runBoundedOperation(mode,argumentsList,{environment=process.env,run=spawnSync,now=Date.now()}={}) {
  assert(['build','setup','prepare','pack','publish','cold'].includes(mode));
  const bytes=readFileSync(join(environment.RUNNER_TEMP,'browser-engine-job-clock.json'));
  assert.match(environment.ENGINE_CLOCK_SHA256??'',/^[a-f0-9]{64}$/u,'source-owned first-step clock digest required');
  assert.equal(createHash('sha256').update(bytes).digest('hex'),environment.ENGINE_CLOCK_SHA256,'first-step clock bytes changed');
  const clock=JSON.parse(bytes);
  const seconds=Math.min(remainingSeconds(clock,environment,now),mode==='cold'?ENGINE_COLD_SECONDS:['setup','build'].includes(mode)?295:1195);
  const result=run('timeout',deadlineArguments(seconds,argumentsList),{stdio:'inherit',env:environment});
  if(result.error)throw result.error;
  assert.equal(result.signal,null,'bounded process interrupted');
  assert(Number.isInteger(result.status),'bounded process has no exit status');return result.status;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href) {
  process.exitCode=runBoundedOperation(process.argv[2],process.argv.slice(3));
}
