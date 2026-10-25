#!/usr/bin/env node
// Finalize only the exact completed owner run; never infer ownership from labels alone.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createEvidenceClient} from './collect-test-replays.mjs';
import {cleanupGeneratedIssues} from './close-finished-test-issues.mjs';
import {custodyFromLog, effectWasNotStarted, originalWorkflow, generatedIssue} from './lib/generated-task-custody.mjs';
import {verifyIsolatedCleanup, requireIsolatedOwner, selectIsolatedFixture,
  cleanupIsolatedTask} from './lib/hive-isolated-task.mjs';
import {redact} from './collect-ci-failures.mjs';
const repository = 'link-assistant/formal-ai';
export async function finalizeGeneratedRun({runId, apply = false, expectedRunAttempt = null,
  expectedProtocolHead = null, client = createEvidenceClient({
  bounds: {retries: 0, logBytes: 1048576, logDownloadBytes: 8388608}}),
  cleanup = cleanupGeneratedIssues} = {}) {
  if (!Number.isSafeInteger(runId) || runId < 1) throw new Error('Invalid owner run identifier');
  if (expectedRunAttempt !== null && (!Number.isSafeInteger(expectedRunAttempt) || expectedRunAttempt < 1)
    || expectedProtocolHead !== null && !/^[a-f0-9]{40}$/u.test(expectedProtocolHead)) throw new Error('Invalid expected owner identity');
  const run = await client.json(`repos/${repository}/actions/runs/${runId}`);
  if (expectedRunAttempt !== null && run.run_attempt !== expectedRunAttempt
    || expectedProtocolHead !== null && run.head_sha !== expectedProtocolHead) {
    const error = new Error('Owner identity changed before cleanup');
    error.code = 'OWNER_CHANGED_BEFORE_CLEANUP'; throw error;
  }
  if (run.id !== runId || run.repository?.full_name !== repository
    || run.path?.split('@')[0] !== originalWorkflow || run.status !== 'completed'
    || !/^[a-f0-9]{40}$/u.test(run.head_sha ?? '')
    || !Number.isSafeInteger(run.run_attempt) || run.run_attempt < 1) throw new Error('Unproved completed owner run');
  const listing = await client.json(`repos/${repository}/actions/runs/${runId}/jobs?per_page=100&filter=latest`);
  if (!Array.isArray(listing.jobs) || listing.total_count > 100
    || listing.jobs.length !== listing.total_count
    || listing.jobs.some(job => job.status !== 'completed')) throw new Error('Incomplete terminal owner jobs');
  const jobs = listing.jobs.filter(job => /^Actual Hive (?:claude|codex|agent) original delivery$/u.test(job.name));
  if (jobs.length > 3 || new Set(jobs.map(job => job.name)).size !== jobs.length) throw new Error('Ambiguous owner client jobs');
  const candidates = [], discovery = [];
  for (const job of jobs) {
    const log = await client.request(`repos/${repository}/actions/jobs/${job.id}/logs`, {log: true, full: true});
    const custody = custodyFromLog({repository, run, job, log});
    if (effectWasNotStarted(job, custody)) {
      discovery.push({job: job.id, status: 'not-started', reason: 'no-task-effects-started'}); continue;
    }
    if (!custody.complete || custody.issueNumbers.length !== 1 || custody.finalIssueUrls.length !== 1
      || custody.finalIssueUrls[0] !== `https://github.com/${repository}/issues/${custody.issueNumbers[0]}`) {
      discovery.push({job: job.id, status: 'retained', reason: 'incomplete-or-ambiguous-owned-log',
        evidence: {complete: custody.complete, issueCount: custody.issueNumbers.length,
          finalUrlCount: custody.finalIssueUrls.length, logOk: log.ok === true,
          streamComplete: log.streamComplete === true, tailComplete: log.tailComplete === true,
          truncated: log.truncated === true, downloadTruncated: log.downloadTruncated === true,
          timedOut: log.timedOut === true, error: redact(log.error ?? '').slice(0, 500)}}); continue;
    }
    if (custody.isolatedFixtures) {
      try {
        const fixture = selectIsolatedFixture(custody.isolatedFixtures);
        if (fixture?.resource?.issueNumber !== custody.issueNumbers[0]
          || fixture.resource.issueUrl !== custody.finalIssueUrls[0]) throw new Error('Isolated cleanup owner differs');
        const context = {repository, run, job, client, recognizeIssue: generatedIssue};
        await requireIsolatedOwner(fixture, context);
        if (!apply) {
          discovery.push({job: job.id, status: 'would-close-owned-fixture'}); continue;
        }
        let writes = 0;
        const api = async (endpoint, {method = 'GET', body} = {}) => {
          if (!endpoint.startsWith('repos/' + repository + '/')) throw new Error('Foreign fixture endpoint');
          if (method === 'GET') return client.json(endpoint);
          if (!['POST', 'PATCH', 'DELETE'].includes(method) || ++writes > 8) throw new Error('Fixture mutation budget');
          const mutation = ['--method', method];
          for (const [key, value] of Object.entries(body ?? {})) mutation.push('-f', key + '=' + value);
          const result = await client.request(endpoint, {mutation});
          if (!result.ok || result.truncated || result.downloadTruncated || result.timedOut) {
            throw new Error('Fixture write uncertain; no retry permitted');
          }
          return result.text?.trim() ? JSON.parse(result.text) : null;
        };
        await cleanupIsolatedTask(fixture, {api, save() {}, outcome: job.conclusion});
        await verifyIsolatedCleanup(fixture, context);
        discovery.push({job: job.id, status: 'closed-owned-fixture', comment: 'recorded', fixture});
      } catch (error) {
        discovery.push({job: job.id, status: 'retained', reason: redact(error.message)});
      }
      continue;
    }
    candidates.push({number: custody.issueNumbers[0], runAttempt: run.run_attempt,
      expectedLogSha256: custody.logSha256, proof: {run: runId, job: job.id, head: run.head_sha}});
  }
  if (new Set(candidates.map(row => row.number)).size !== candidates.length) throw new Error('Ambiguous shared issue custody');
  const prepared = await cleanup({plan: {repository, candidates}, client, allowOwnedCancelled: true});
  const applied = apply ? await cleanup({plan: prepared, client, apply: true, allowOwnedCancelled: true}) : null;
  const perTool = jobs.map(job => {
    const candidate = candidates.find(row => row.proof.job === job.id);
    const result = applied?.results.find(row => row.number === candidate?.number);
    const alreadyFinalized = discovery.find(row => row.job === job.id && row.status === 'closed-owned-fixture');
    const closed = result?.status === 'closed' || !!alreadyFinalized;
    const notStarted = discovery.some(row => row.job === job.id && row.status === 'not-started');
    const comment = alreadyFinalized?.comment ?? result?.comment ?? 'not-confirmed';
    const complete = closed && comment === 'recorded';
    const retained = discovery.find(row => row.job === job.id && row.status === 'retained');
    return {tool: job.name.split(' ')[2], job: job.id, outcome: job.conclusion, complete,
      closed: closed ? 1 : 0, retained: closed || notStarted ? 0 : 1, notStarted,
      comment, ...(retained ? {retention: {reason: retained.reason, ...(retained.evidence ? {evidence: retained.evidence} : {})}} : {})};
  });
  const cleanupReceipt = {schema: 'generated-e2e-cleanup-proof/v1', sourceRun: runId,
    sourceAttempt: run.run_attempt, protocolHead: run.head_sha, outcome: run.conclusion,
    complete: apply && perTool.some(row => row.complete) && perTool.every(row => row.complete || row.notStarted), perTool,
    noEffects: apply && jobs.length > 0 && perTool.every(row => row.notStarted),
    closed: perTool.reduce((sum, row) => sum + row.closed, 0),
    retained: perTool.reduce((sum, row) => sum + row.retained, 0)};
  // protocolHead identifies the owning workflow, not a separately admitted tested PR head.
  // Artifact custody must authenticate this receipt's producer; JSON alone is not authority.
  return {schema: 'generated-e2e-finalization/v1', run: runId, sourceHead: run.head_sha,
    runAttempt: run.run_attempt, apply, discovery, prepared, applied, cleanupReceipt};
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const args = process.argv.slice(2);
  if (args.length < 1 || args.length > 2 || !/^[1-9]\d*$/u.test(args[0])
    || args.length === 2 && args[1] !== '--apply') throw new Error('Usage: finalize-generated-test-issues.mjs RUN_ID [--apply]');
  const apply = args.includes('--apply');
  const lock = path.join(os.tmpdir(), 'formal-ai-generated-e2e-cleanup.lock');
  let descriptor;
  try {
    if (apply) descriptor = fs.openSync(lock, 'wx', 0o600);
    const result = await finalizeGeneratedRun({runId: Number(args[0]), apply});
    process.stdout.write(JSON.stringify(result, null, 2) + '\n');
    if (result.applied?.results.some(row => row.status === 'mutation-uncertain'
      || row.comment === 'uncertain-not-retried')) process.exitCode = 1;
  } catch (error) { process.stderr.write(redact(error.message) + '\n'); process.exitCode = 1; }
  finally { if (descriptor !== undefined) { fs.closeSync(descriptor); fs.unlinkSync(lock); } }
}
