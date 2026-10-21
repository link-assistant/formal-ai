#!/usr/bin/env node
// Dry-run first. Each authorized mutation rechecks the exact generated issue and owner run.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createEvidenceClient} from './collect-test-replays.mjs';
import {redact} from './collect-ci-failures.mjs';
import {cleanupDecision, custodyFromLog, generatorFromSourceInputs, issueSnapshot, sameIssueSnapshot} from './lib/generated-task-custody.mjs';

const repository = 'link-assistant/formal-ai';
let applyActive = false;
export async function cleanupGeneratedIssues({plan, apply = false, comment = true,
  allowOwnedCancelled = false, client = createEvidenceClient({bounds: {retries: 0,
    logBytes: 1048576, logDownloadBytes: 8388608}})} = {}) {
  const candidates = plan?.eligibleClosures ?? plan?.candidates;
  if (!Array.isArray(candidates) || candidates.length > 100 || candidates.some(row =>
    !Number.isSafeInteger(row.number) || row.number < 1)) throw new Error('Invalid bounded cleanup plan');
  if (plan.repository && plan.repository !== repository) throw new Error('Foreign cleanup repository');
  if (apply && candidates.some(row => !row.expectedIssue || !Number.isSafeInteger(row.runAttempt)
    || row.runAttempt < 1 || !/^[a-f0-9]{64}$/u.test(row.expectedLogSha256 ?? ''))) throw new Error('Apply requires reviewed dry-run issue, run and log snapshots');
  if (apply && applyActive) throw new Error('Concurrent cleanup apply refused');
  if (apply) applyActive = true;
  try {
  const results = [];
  for (const candidate of candidates) {
    const proof = candidate.proof ?? candidate.proofs?.[0];
    if (!proof || !Number.isSafeInteger(proof.run) || !Number.isSafeInteger(proof.job)) throw new Error('Missing owned run/job');
    let verifiedCustody = null;
    async function fresh(expected) {
      const issue = await client.json(`repos/${repository}/issues/${candidate.number}`);
      if (expected && !sameIssueSnapshot(expected, issue)) return {eligible: false, reason: 'issue-changed-since-reviewed-plan'};
      const run = await client.json(`repos/${repository}/actions/runs/${proof.run}`);
      if (candidate.runAttempt && candidate.runAttempt !== run.run_attempt) return {eligible: false, reason: 'owner-run-attempt-changed'};
      const listing = await client.json(`repos/${repository}/actions/runs/${proof.run}/jobs?per_page=100&filter=latest`);
      if (listing.total_count > 100 || listing.jobs.length !== listing.total_count) return {eligible: false, reason: 'incomplete-owner-jobs'};
      const job = listing.jobs.find(row => row.id === proof.job);
      if (!job) return {eligible: false, reason: 'owner-job-not-in-current-attempt'};
      if (!verifiedCustody) {
        const source = await client.json(`repos/${repository}/contents/data/benchmarks/hive-original-delivery-inputs.json?ref=${run.head_sha}`);
        if (source.encoding !== 'base64' || typeof source.content !== 'string' || source.size > 65536) return {eligible: false, reason: 'missing-actual-generator-inputs'};
        const encoded = source.content.replace(/\s+/gu, ''), bytes = Buffer.from(encoded, 'base64');
        if (bytes.toString('base64') !== encoded || bytes.length !== source.size) return {eligible: false, reason: 'invalid-generator-source-bytes'};
        const generator = generatorFromSourceInputs(bytes.toString('utf8'), run.head_sha);
        if (!generator) return {eligible: false, reason: 'unknown-actual-generator-source'};
        const log = await client.request(`repos/${repository}/actions/jobs/${job.id}/logs`, {log: true, full: true});
        verifiedCustody = custodyFromLog({repository, run, job, log, generator});
      }
      if (candidate.expectedLogSha256 && candidate.expectedLogSha256 !== verifiedCustody.logSha256) return {eligible: false, reason: 'custody-log-changed-since-reviewed-plan'};
      const decision = cleanupDecision({repository, issue, run, jobs: listing.jobs,
        custody: verifiedCustody, expectedSourceHead: proof.head ?? proof.sourceHead,
        allowOwnedCancelled});
      if (decision.eligible) {
        // Optimistic revalidation, not an atomic server-side conditional mutation.
        const finalIssue = await client.json(`repos/${repository}/issues/${candidate.number}`);
        if (!sameIssueSnapshot(decision.snapshot, finalIssue)) return {eligible: false,
          reason: 'issue-changed-during-owner-proof-collection'};
      }
      return {...decision, runAttempt: run.run_attempt};
    }
    let decision;
    try { decision = await fresh(candidate.expectedIssue); }
    catch (error) { results.push({number: candidate.number, status: 'retained', reason: redact(error.message)}); continue; }
    if (!decision.eligible) { results.push({number: candidate.number, status: 'retained', reason: decision.reason}); continue; }
    const prepared = {number: candidate.number, expectedIssue: decision.snapshot,
      runAttempt: decision.runAttempt, expectedLogSha256: verifiedCustody.logSha256,
      proof: {run: proof.run, job: proof.job, head: decision.sourceHead},
      custody: verifiedCustody, decision};
    if (!apply) { results.push({...prepared, status: 'would-close'}); continue; }
    // The final issue GET completed after all proof reads. This remains optimistic revalidation.
    let closed;
    try {
      closed = await client.request(`repos/${repository}/issues/${candidate.number}`,
        {mutation: ['--method', 'PATCH', '-f', 'state=closed', '-f', `state_reason=${decision.reason}`]});
    } catch (error) {
      results.push({...prepared, status: 'mutation-uncertain', reason: redact(error.message)}); break;
    }
    if (!closed.ok) {
      results.push({...prepared, status: 'mutation-uncertain', reason: 'Close request not retried; inspect remote state'});
      break;
    }
    let closedIssue;
    try { closedIssue = JSON.parse(closed.text); } catch { closedIssue = null; }
    const closedSnapshot = closedIssue && issueSnapshot(closedIssue);
    if (!closedSnapshot || closedSnapshot.state !== 'closed'
      || ['number', 'author', 'title', 'createdAt', 'bodySha256', 'comments']
        .some(key => closedSnapshot[key] !== decision.snapshot[key])
      || JSON.stringify(closedSnapshot.labels) !== JSON.stringify(decision.snapshot.labels)) {
      results.push({...prepared, status: 'mutation-uncertain', reason: 'Close receipt did not prove expected issue state'});
      break;
    }
    const result = {...prepared, status: 'closed', comment: 'not-requested'};
    results.push(result);
    if (comment) {
      try {
      const run = await client.json(`repos/${repository}/actions/runs/${proof.run}`);
      const jobs = await client.json(`repos/${repository}/actions/runs/${proof.run}/jobs?per_page=100&filter=latest`);
      const current = await client.json(`repos/${repository}/issues/${candidate.number}`);
      if (!sameIssueSnapshot(closedSnapshot, current) || run.status !== 'completed'
        || run.head_sha !== decision.sourceHead || run.run_attempt !== decision.runAttempt
        || jobs.total_count > 100 || jobs.jobs.length !== jobs.total_count
        || jobs.jobs.some(job => job.status !== 'completed')) {
        result.comment = 'retained-changed-state'; continue;
      }
      const body = `<!-- generated-e2e-cleanup run=${proof.run} job=${proof.job} -->\n`
        + `Finished generated test fixture. Outcome: ${decision.outcome}. Evidence: ${decision.runUrl}\n`
        + 'This closure retires the test fixture and does not certify a user feature.';
      const posted = await client.request(`repos/${repository}/issues/${candidate.number}/comments`,
        {mutation: ['--method', 'POST', '-f', `body=${body}`]});
      result.comment = posted.ok ? 'recorded' : 'uncertain-not-retried';
      if (!posted.ok) break;
      } catch (error) {
        result.comment = 'uncertain-not-retried'; result.commentError = redact(error.message); break;
      }
    }
  }
  return {schema: 'generated-e2e-cleanup-receipt/v1', repository, apply,
    candidates: results.filter(row => row.status === 'would-close').map(({number, expectedIssue, runAttempt, expectedLogSha256, proof}) =>
      ({number, expectedIssue, runAttempt, expectedLogSha256, proof})), results, requests: client.attempts,
    transferredBytes: client.transferredBytes()};
  } finally { if (apply) applyActive = false; }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const args = process.argv.slice(2);
  if (args.length < 1 || args.slice(1).some(value => value !== '--apply' && value !== '--no-comment')) throw new Error('Usage: close-finished-test-issues.mjs PLAN [--apply] [--no-comment]');
  const apply = args.includes('--apply'), file = path.join(os.tmpdir(), 'formal-ai-generated-e2e-cleanup.lock');
  let descriptor;
  try {
    if (apply) descriptor = fs.openSync(file, 'wx', 0o600);
    const stat = fs.statSync(args[0]);
    if (!stat.isFile() || stat.size > 5242880) throw new Error('Cleanup plan exceeds finite file bound');
    const plan = JSON.parse(fs.readFileSync(args[0], 'utf8'));
    await cleanupGeneratedIssues({plan, apply, comment: !args.includes('--no-comment')})
      .then(receipt => {
        process.stdout.write(JSON.stringify(receipt, null, 2) + '\n');
        if (receipt.results.some(row => row.status === 'mutation-uncertain' || row.comment === 'uncertain-not-retried')) process.exitCode = 1;
      });
  } catch (error) { process.stderr.write(redact(error.message) + '\n'); process.exitCode = 1; }
  finally { if (descriptor !== undefined) { fs.closeSync(descriptor); fs.unlinkSync(file); } }
}
