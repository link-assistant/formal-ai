import test from 'node:test';
import assert from 'node:assert/strict';
import { collect, diagnostics, deduplicate, normalize, redact, observedOutcome } from '../../../scripts/collect-ci-failures.mjs';

test('fingerprints collapse volatile identities but preserve error codes and argument names', () => {
  const a = diagnostics('2026-10-19T01:02:03Z Error invalid arguments uuid=12345678-1234-1234-1234-123456789abc /tmp/job-a EINVAL --head');
  const b = diagnostics('2026-10-20T02:03:04Z Error invalid arguments uuid=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee /tmp/job-b EINVAL --head');
  assert.equal(a[0].fingerprint, b[0].fingerprint);
  assert.notEqual(a[0].fingerprint, diagnostics('Error invalid arguments uuid=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee /tmp/job-b EACCES --base')[0].fingerprint);
  assert.equal(deduplicate([{ repository: 'a/b', diagnostics: a }, { repository: 'a/c', diagnostics: b }])[0].occurrences.length, 2);
});
test('non-finite and excessive bounds are rejected before IO', async () => {
  let calls = 0;
  for (const value of [Infinity, NaN, 0, -1, 1.5, 101]) {
    await assert.rejects(collect({ bounds: { runs: value }, transport: async () => { calls++; } }), /Invalid runs/u);
  }
  assert.equal(calls, 0);
});
test('partial logs retain diagnostics, unavailable logs and cancelled/skipped runs remain explicit', async () => {
  const result = await collect({ repositories: ['a/b'], bounds: { retries: 0 }, transport: async ([, endpoint]) => {
    if (endpoint.includes('/runs?')) return { ok: true, text: JSON.stringify({ workflow_runs: [{ id: 1, conclusion: 'failure' }, { id: 2, conclusion: 'cancelled' }, { id: 3, conclusion: 'skipped' }] }) };
    if (endpoint.includes('/jobs?')) return { ok: true, text: JSON.stringify({ total_count: 10, jobs: [{ id: 4, conclusion: 'failure' }, { id: 5, conclusion: 'failure' }] }) };
    if (endpoint.includes('/4/')) return { ok: false, truncated: true, text: 'Error invalid arguments --command' };
    return { ok: false, error: 'HTTP 404' };
  } });
  assert.equal(result.cancelled.length, 1);
  assert.equal(result.skipped.length, 1);
  assert.equal(result.failures[0].jobsTruncated, true);
  assert.equal(result.failures[0].jobs[0].logsTruncated, true);
  assert.equal(result.failures[0].jobs[0].logsUnavailable, false);
  assert.equal(result.failures[0].jobs[1].logsUnavailable, true);
  assert.equal(result.groups.length, 1);
});
test('retries terminate and forbidden credentials do not appear in diagnostics', async () => {
  let calls = 0;
  const result = await collect({ repositories: ['a/b'], bounds: { retries: 2 }, transport: async () => { calls++; return { ok: false, timedOut: true, error: 'timeout' }; } });
  assert.equal(calls, 3);
  assert.equal(result.unavailable[0].attempts, 3);
  const safe = redact('token=ghp_abcdefghijkl password=hello user@example.com /Users/alice/x 123456789012 10.1.2.3');
  assert.doesNotMatch(safe, /abcdefghijkl|hello|alice|example.com|123456789012|10\.1\.2\.3/u);
  assert.match(normalize('Error EINVAL --command'), /einval --command/u);
});

test('transport retains actual start and tail with finite transfer and time budgets', async () => {
  const { mkdtempSync, writeFileSync, rmSync } = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { runGh } = await import('../../../scripts/collect-ci-failures.mjs');
  const directory = mkdtempSync(join(tmpdir(), 'ci-log-fixture-'));
  const previous = process.env.PATH;
  try {
    writeFileSync(join(directory, 'gh'), `#!${process.execPath}\nprocess.stdout.write('START\\n'+'x'.repeat(2000)+'\\nError EINVAL --command TAIL\\n');\n`, { mode: 0o755 });
    process.env.PATH = directory + ':' + previous;
    const complete = await runGh([], { timeoutMs: 2000, maxBytes: 128, maxDownloadBytes: 4096 });
    assert.equal(complete.ok, true);
    assert.equal(complete.truncated, true);
    assert.equal(complete.tailComplete, true);
    assert.match(complete.text, /^START/u);
    assert.match(complete.text, /EINVAL --command TAIL/u);
    const capped = await runGh([], { timeoutMs: 2000, maxBytes: 64, maxDownloadBytes: 128 });
    assert.equal(capped.downloadTruncated, true);
    assert.equal(capped.tailComplete, false);
    assert.equal(capped.bytesReceived, 128);
  } finally { process.env.PATH = previous; rmSync(directory, { recursive: true, force: true }); }
});

test('quoted agent payloads and echoed configuration cannot masquerade as observed errors', () => {
  assert.equal(diagnostics('"bodyPreview": "Error invalid arguments --command"\nif-no-files-found: error\n[verbose] 0 updated, 1 current, 0 failed').length, 0);
  assert.equal(diagnostics('Error: Failed to fetch authenticated GitHub username: gh: Resource not accessible by integration (HTTP 403)')[0].category, 'github-integration-permission');
});

test('semantic source research reads declared prerequisites and never grants unproved writes', async () => {
  const { mkdtempSync, writeFileSync, rmSync, existsSync } = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { drive } = await import('../../../experiments/js_dogfood/drive.mjs');
  const { planChatStep } = await import('../../../js/agentic/planner.mjs');
  const directory = mkdtempSync(join(tmpdir(), 'ci-source-research-'));
  const prompt = 'Write a function identity(value) that preserves its input in identity.mjs, add a test in identity.test.mjs, and run node --test identity.test.mjs.';
  try {
    writeFileSync(join(directory, 'identity.test.mjs'), "import {identity} from './identity.mjs';\nimport assert from 'node:assert/strict';\nassert.equal(identity('unseen-719'), 'unseen-719');\n");
    const actual = await drive(planChatStep, directory, prompt, { steps: 5 });
    assert.equal(actual.stop, 'final');
    assert.deepEqual(actual.transcript.map(item => item.tool), ['read', 'read']);
    assert.match(actual.answer, /SearchToolMissing/u);
    assert.equal(existsSync(join(directory, 'identity.mjs')), false);
    const injected = await drive(async () => ({ kind: 'tool_calls', calls: [{ tool: 'write', arguments: JSON.stringify({ filePath: 'foreign.mjs', content: 'export const forged = true;\n' }) }] }), directory, prompt, { steps: 2 });
    assert.equal(injected.stop, 'unbound-research-prerequisite');
    assert.equal(existsSync(join(directory, 'foreign.mjs')), false);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('quoted credentials, signed links and unknown bounds fail closed', async () => {
  assert.doesNotMatch(redact('Error {"password":"private-value","token":"private-token"}'), /private-value|private-token/u);
  assert.equal(redact('https://blob.example/file?sig=private-signature'), '[signed-url]');
  let calls=0;
  await assert.rejects(collect({bounds:{unknown:Infinity},transport:async()=>{calls++;}}), /Unknown bounds/u);
  assert.equal(calls,0);
});

test('bounded diagnostic retention still examines the actual tail', () => {
  const log=Array.from({length:70},(_,i)=>'Error ordinary failure '+i).join('\n')+'\nError invalid arguments --command';
  const result=diagnostics(log);
  assert.equal(result.length,40);
  assert(result.some(x=>x.category==='invalid-action-arguments'));
});

test('assertion classes and meaningful error codes remain reviewable', () => {
  const result=diagnostics('AssertionError [ERR_ASSERTION]: requested pull-request projection source is unsupported without authenticated merge selection');
  assert.equal(result.length,1);
  assert.match(result[0].diagnostic, /err_assertion/u);
});


test('recovered upload warnings cannot hide observed absent source delivery', () => {
  const log = 'Error: Failed to fetch authenticated GitHub username: HTTP 403\n'
    + 'Preserving the complete sanitized log on the pull request branch\n'
    + 'Log URL: https://github.com/owner/repo/blob/commit/dev/log/solve.log\n'
    + '⚠️ Finished without changes\n'
    + '##[error]Error: Failed to fetch authenticated GitHub username: HTTP 403\n';
  assert.deepEqual(observedOutcome(log), {sourceDelivery:'no-changes-observed', logPublication:'recovered-after-permission-denial'});
  assert(diagnostics(log).some(item => item.category === 'no-delivered-changes'));
  assert.deepEqual(observedOutcome('"output": "Finished without changes"\n"bodyPreview": "Log URL: https://github.com/owner/repo/blob/hash/log"'), {sourceDelivery:'unknown', logPublication:'unknown'});
  assert.equal(observedOutcome('Error Resource not accessible by integration').logPublication, 'unknown');
});

test('gh timeout kills its owned process group and observes inherited stream closure', { skip: process.platform === 'win32' }, async () => {
  const fs = await import('node:fs');
  const os = await import('node:os');
  const path = await import('node:path');
  const { runGh } = await import('../../../scripts/collect-ci-failures.mjs');
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'ci-gh-group-'));
  const previous = process.env.PATH;
  try {
    fs.writeFileSync(path.join(directory, 'gh'), `#!${process.execPath}\nconst {spawn}=require('node:child_process');\nspawn(process.execPath,['-e','setTimeout(()=>{},10000)'],{stdio:['ignore','inherit','inherit']});\nprocess.stdout.write('START\\n');\nsetTimeout(()=>{},10000);\n`, { mode: 0o755 });
    process.env.PATH = directory + path.delimiter + previous;
    const started = Date.now();
    const result = await runGh([], { timeoutMs: 1000, maxBytes: 128, maxDownloadBytes: 1024 });
    assert.ok(Date.now() - started < 3500, 'timeout must include finite post-kill settlement');
    assert.equal(result.ok, false);
    assert.equal(result.timedOut, true);
    assert.equal(result.processClosed, true);
    assert.equal(result.streamComplete, true);
    assert.equal(result.cleanupComplete, true);
    assert.equal(result.tailComplete, false);
    assert.match(result.text, /^START/u);
  } finally { process.env.PATH = previous; fs.rmSync(directory, { recursive: true, force: true }); }
});

test('gh exit cannot cancel the deadline while an escaped descendant holds its pipes', { skip: process.platform === 'win32' }, async () => {
  const fs = await import('node:fs');
  const os = await import('node:os');
  const path = await import('node:path');
  const { runGh } = await import('../../../scripts/collect-ci-failures.mjs');
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'ci-gh-held-pipe-'));
  const previous = process.env.PATH;
  const descendant = path.join(directory, 'descendant.json');
  try {
    fs.writeFileSync(path.join(directory, 'gh'), `#!${process.execPath}\nconst fs=require('node:fs'),{spawn}=require('node:child_process');\nconst child=spawn(process.execPath,['-e','setTimeout(()=>{},10000)'],{detached:true,stdio:['ignore','inherit','inherit']});\nfs.writeFileSync(${JSON.stringify(descendant)},JSON.stringify({pid:child.pid}));\nprocess.stdout.write('START\\n',()=>process.exit(0));\n`, { mode: 0o755 });
    process.env.PATH = directory + path.delimiter + previous;
    const started = Date.now();
    const result = await runGh([], { timeoutMs: 1000, maxBytes: 128, maxDownloadBytes: 1024 });
    assert.ok(Date.now() - started < 3500, 'escaped pipe must not outlive the settlement bound');
    assert.equal(result.ok, false);
    assert.equal(result.timedOut, true);
    assert.equal(result.exitCode, 0, 'direct child really exited before its inherited pipes closed');
    assert.equal(result.streamComplete, false);
    assert.equal(result.cleanupComplete, false, 'forced pipe destruction cannot assert cleanup');
    assert.equal(result.tailComplete, false);
    assert.match(result.text, /^START/u);
  } finally {
    if (fs.existsSync(descendant)) {
      const { pid } = JSON.parse(fs.readFileSync(descendant, 'utf8'));
      try { process.kill(pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
    }
    process.env.PATH = previous;
    fs.rmSync(directory, { recursive: true, force: true });
  }
});


function negativeTransport(runs, jobsForAttempt, logs, endpoints) {
  return async ([, endpoint]) => {
    endpoints.push(endpoint);
    if (endpoint.includes('/runs?')) return {ok:true,text:JSON.stringify({workflow_runs:runs})};
    if (endpoint.includes('/jobs?')) {
      const match = endpoint.match(/\/attempts\/(\d+)\/jobs/u);
      const jobs = jobsForAttempt(match ? Number(match[1]) : 1, endpoint);
      return {ok:true,text:JSON.stringify({total_count:jobs.length,jobs})};
    }
    return logs(endpoint);
  };
}
const executedNegativeJob = (id, conclusion) => ({id,conclusion,status:'completed',started_at:'2026-01-01T00:00:00Z',
  runner_id:7,steps:[{number:1,name:'assertions',status:'completed',conclusion:'failure',started_at:'2026-01-01T00:00:01Z'}]});
const completeLog = text => ({ok:true,text,tailComplete:true,bytesReceived:Buffer.byteLength(text)});
test('cancelled run retains earlier assertion and later cancellation without changing conclusion', async () => {
  const endpoints=[];
  const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:negativeTransport(
    [{id:11,run_attempt:1,conclusion:'cancelled'}],()=>[executedNegativeJob(21,'cancelled')],
    ()=>completeLog('AssertionError: expected true\nThe operation was canceled.\n'),endpoints)});
  assert.equal(report.failures.length,0);assert.equal(report.cancelled.length,1);
  assert.equal(report.cancelled[0].jobs[0].lifecycle.execution,'execution-observed');
  assert.equal(report.cancelled[0].jobs[0].jobConclusion,'cancelled');
  assert(report.groups.some(group=>group.diagnostic.includes('assertionerror')));
  assert(report.groups.some(group=>group.diagnostic.includes('canceled')));
  assert.equal(endpoints.filter(endpoint=>endpoint.endsWith('/logs')).length,1);
  assert.equal(report.cancelled[0].attemptCoverage.complete,true);
});
test('failed run includes started cancelled sibling and terminal other-negative logs', async () => {
  const endpoints=[];const conclusions=['failure','cancelled','timed_out','action_required','stale'];
  const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:negativeTransport(
    [{id:12,run_attempt:1,conclusion:'failure'}],()=>conclusions.map((value,index)=>executedNegativeJob(30+index,value)),
    endpoint=>completeLog('Error distinct '+endpoint),endpoints)});
  assert.equal(report.failures[0].jobs.length,5);assert.equal(endpoints.filter(endpoint=>endpoint.endsWith('/logs')).length,5);
  assert.equal(report.groups.length,5);
});
test('explicit unstarted and unknown lifecycle never become execution from job timestamp alone', async () => {
  const endpoints=[];const jobs=[{id:40,conclusion:'cancelled',started_at:null,runner_id:null,steps:[]},
    {id:41,conclusion:'cancelled',started_at:'2026-01-01T00:00:00Z',runner_id:5}];
  const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:negativeTransport(
    [{id:13,run_attempt:1,conclusion:'cancelled'}],()=>jobs,()=>({ok:false,error:'HTTP 404'}),endpoints)});
  assert.deepEqual(report.cancelled[0].jobs.map(job=>job.lifecycle.execution),['explicitly-unstarted','unknown-start']);
  for(const job of report.cancelled[0].jobs){assert.equal(job.logsUnavailable,true);assert.equal(job.logAvailability,'unavailable');assert.match(job.logError,/404/u);}
  assert.equal(endpoints.filter(endpoint=>endpoint.endsWith('/logs')).length,2);
});
test('negative logs preserve transfer truncation, unknown tails and absent logs distinctly', async () => {
  const endpoints=[];const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:negativeTransport(
    [{id:14,run_attempt:1,conclusion:'stale'}],()=>[executedNegativeJob(50,'stale'),executedNegativeJob(51,'cancelled')],
    endpoint=>endpoint.includes('/50/')?{ok:false,truncated:true,downloadTruncated:true,tailComplete:false,text:'Error before cutoff',bytesReceived:123}
      :{ok:false,text:'',error:'HTTP 404'},endpoints)});
  assert.equal(report.other[0].jobs[0].logsDownloadTruncated,true);assert.equal(report.other[0].jobs[0].logsTailComplete,false);
  assert.equal(report.other[0].jobs[0].logsUnavailable,false);assert.equal(report.other[0].jobs[1].logsUnavailable,true);
  assert.equal(report.groups.length,1);
});
test('earlier failed attempt survives a successful latest rerun and has attempt-specific custody', async () => {
  const endpoints=[];const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:negativeTransport(
    [{id:15,run_attempt:2,conclusion:'success'}],attempt=>[executedNegativeJob(60+attempt,attempt===2?'success':'failure')],
    ()=>completeLog('Error earlier actual attempt'),endpoints)});
  assert.deepEqual(report.other[0].jobs.map(job=>[job.runAttempt,job.jobConclusion]),[[2,'success'],[1,'failure']]);
  assert.equal(report.other[0].attemptCoverage.complete,true);assert.equal(report.groups[0].occurrences[0].runAttempt,1);
  assert.equal(endpoints.filter(endpoint=>endpoint.endsWith('/logs')).length,1);
});
test('attempt and aggregate job caps expose incomplete coverage without inventing completeness', async () => {
  const endpoints=[];const report=await collect({repositories:['a/b'],bounds:{retries:0,pages:1,jobs:1},transport:negativeTransport(
    [{id:16,run_attempt:3,conclusion:'cancelled'}],()=>[executedNegativeJob(70,'cancelled')],()=>completeLog('Error bounded'),endpoints)});
  assert.equal(report.cancelled[0].attemptCoverage.complete,false);assert.deepEqual(report.cancelled[0].attemptCoverage.considered,[3]);
  assert(report.cancelled[0].attemptCoverage.reasons.includes('attempt-limit'));assert.equal(endpoints.length,3);
});


test('unknown or invalid attempt metadata keeps latest endpoint without inventing attempt one', async () => {
  for (const runAttempt of [undefined, null, 0, -1, 1.5, '2']) {
    const endpoints=[];
    const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:negativeTransport(
      [{id:80,run_attempt:runAttempt,conclusion:'cancelled'}],()=>[executedNegativeJob(81,'cancelled')],
      ()=>completeLog('Error unknown attempt'),endpoints)});
    const entry=report.cancelled[0];
    assert.equal(entry.jobs[0].runAttempt,null);
    assert.equal(entry.attempts[0].attempt,null);
    assert.deepEqual(entry.attemptCoverage.considered,[null]);
    assert.equal(entry.attemptCoverage.reportedAttempts,null);
    assert.equal(entry.attemptCoverage.complete,false);
    assert(entry.attemptCoverage.reasons.includes('attempt-count-unavailable'));
    assert(endpoints.some(endpoint=>endpoint.includes('/runs/80/jobs?')));
    assert(!endpoints.some(endpoint=>endpoint.includes('/attempts/')));
    assert.equal(report.groups[0].occurrences[0].runAttempt,null);
  }
});
test('completed step timestamps with absent or unknown conclusion do not establish execution', async () => {
  const endpoints=[];
  const conclusions=[undefined,null,'unrecognized','skipped','success','failure'];
  const jobs=conclusions.map((conclusion,index)=>({id:90+index,conclusion:'cancelled',status:'completed',
    started_at:'2026-01-01T00:00:00Z',runner_id:7,steps:[{number:1,status:'completed',
      started_at:'2026-01-01T00:00:01Z',...(conclusion===undefined?{}:{conclusion})}]}));
  const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:negativeTransport(
    [{id:89,run_attempt:1,conclusion:'cancelled'}],()=>jobs,()=>completeLog('Error retained'),endpoints)});
  assert.deepEqual(report.cancelled[0].jobs.map(job=>job.lifecycle.execution),
    ['unknown-start','unknown-start','unknown-start','unknown-start','execution-observed','execution-observed']);
});
test('short or malformed job inventories are incomplete even below the configured budget', async () => {
  for (const data of [{total_count:2,jobs:[executedNegativeJob(101,'failure')]},
    {total_count:1,jobs:[]},{total_count:0,jobs:null},{total_count:'1',jobs:[]},
    {total_count:0,jobs:[executedNegativeJob(101,'failure')]}]) {
    const report=await collect({repositories:['a/b'],bounds:{retries:0},transport:async([,endpoint])=>{
      if(endpoint.includes('/runs?'))return {ok:true,text:JSON.stringify({workflow_runs:[{id:100,run_attempt:1,conclusion:'failure'}]})};
      if(endpoint.includes('/jobs?'))return {ok:true,text:JSON.stringify(data)};
      return completeLog('Error retained without full inventory');
    }});
    assert.equal(report.failures[0].jobsTruncated,true);
    assert.equal(report.failures[0].attemptCoverage.complete,false);
    assert(report.failures[0].attemptCoverage.reasons.includes('job-list-incomplete'));
  }
});
test('aggregate jobs budget stops older attempts without resetting at each rerun', async () => {
  const endpoints=[];
  const report=await collect({repositories:['a/b'],bounds:{retries:0,jobs:2,pages:5},transport:negativeTransport(
    [{id:110,run_attempt:3,conclusion:'failure'}],attempt=>[executedNegativeJob(110+attempt,'failure')],
    ()=>completeLog('Error token=ghp_abcdefghijkl user@example.com'),endpoints)});
  const entry=report.failures[0];
  assert.deepEqual(entry.jobs.map(job=>job.runAttempt),[3,2]);
  assert.deepEqual(entry.attemptCoverage.considered,[3,2]);
  assert.equal(entry.attemptCoverage.complete,false);
  assert(entry.attemptCoverage.reasons.includes('job-limit'));
  assert.equal(endpoints.filter(endpoint=>endpoint.endsWith('/logs')).length,2);
  assert(!endpoints.some(endpoint=>endpoint.includes('/attempts/1/')));
  assert.doesNotMatch(JSON.stringify(report),/abcdefghijkl|user@example.com/u);
});
