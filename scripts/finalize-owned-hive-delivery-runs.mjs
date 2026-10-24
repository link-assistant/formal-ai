#!/usr/bin/env node
// Two phases: publish an immutable claim before a close; interruption quarantines that owner.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createEvidenceClient} from './collect-test-replays.mjs';
import {effectOwnerArtifact, cleanupClaimName, validateCleanupCursor, prepareCleanupClaim, revalidateCleanupClaim, applyCleanupClaim,
  readScanCursor, makeScanProgress} from './lib/generated-task-cleanup-journal.mjs';
const repository = 'link-assistant/formal-ai';
/** A completed no-op trigger must not rescan all history or download old journals. */
export async function triggeredHiveOwnerExists(reader, trigger) {
  assert.ok(Number.isSafeInteger(trigger?.id) && trigger.id > 0);
  assert.ok(Number.isSafeInteger(trigger.run_attempt) && trigger.run_attempt > 0);
  assert.match(trigger.head_sha, /^[a-f0-9]{40}$/u);
  const run = await reader.read(`repos/${repository}/actions/runs/${trigger.id}/attempts/${trigger.run_attempt}`);
  assert.equal(run.id, trigger.id); assert.equal(run.run_attempt, trigger.run_attempt);
  assert.equal(run.head_sha, trigger.head_sha); assert.equal(run.status, 'completed');
  assert.equal(run.repository?.full_name, repository); assert.equal(run.head_repository?.full_name, repository);
  assert.equal(run.path?.split('@')[0], '.github/workflows/hive-original-delivery.yml');
  const listing = await reader.read(`repos/${repository}/actions/runs/${run.id}/artifacts?name=${effectOwnerArtifact}&per_page=100`);
  assert.ok(Number.isSafeInteger(listing.total_count) && listing.total_count >= 0 && listing.total_count <= 100);
  assert.ok(Array.isArray(listing.artifacts) && listing.artifacts.length === listing.total_count);
  return listing.artifacts.some(artifact => artifact.name === effectOwnerArtifact);
}
export async function prepareOwnedHiveRuns({reader, producer, maxPages = 3, maxOwners = 3,
  cursor = null, trigger = null, now = Date.now(), ...dependencies} = {}) {
  assert.ok(Number.isSafeInteger(maxPages) && maxPages > 0 && maxPages <= 5, 'finite page bound');
  assert.ok(Number.isSafeInteger(maxOwners) && maxOwners > 0 && maxOwners <= 5, 'finite owner bound');
  assert.ok(Number.isFinite(now)); if (cursor !== null) validateCleanupCursor(cursor);
  assert.equal(typeof reader?.read, 'function');
  if (trigger !== null && !await triggeredHiveOwnerExists(reader, trigger)) {
    return {schema: 'owned-hive-cleanup-preparation/v1', repository, artifactName: effectOwnerArtifact,
      owners: [], retained: [], examined: 0, continuation: null, enumerationComplete: false,
      complete: false, incomplete: false, trigger: {run: trigger.id, attempt: trigger.run_attempt,
        immutableOwner: false}, progress: null, progressName: ''};
  }
  cursor ??= await readScanCursor({reader, now, ...dependencies});
  const owners = [], retained = [], seen = new Map(), cache = new Map();
  let examined = 0, census = null;
  async function receipt(continuation, enumerationComplete) {
    const progress = await makeScanProgress({reader, producer, continuation, now, ...dependencies});
    return {schema: 'owned-hive-cleanup-preparation/v1', repository, artifactName: effectOwnerArtifact,
      owners, retained, examined, startCursor: cursor, continuation, enumerationComplete,
      complete: enumerationComplete && retained.length === 0,
      incomplete: !enumerationComplete || retained.length > 0, progressName: progress.name,
      progress: progress.document, readerStatistics: reader.statistics?.() ?? null};
  }
  for (let page = cursor.page; page < cursor.page + maxPages; page++) {
    const listing = await reader.read(`repos/${repository}/actions/artifacts?name=${effectOwnerArtifact}&per_page=100&page=${page}`);
    assert.ok(Number.isSafeInteger(listing.total_count) && listing.total_count >= 0);
    assert.ok(Array.isArray(listing.artifacts) && listing.artifacts.length <= 100);
    if (census !== null && census !== listing.total_count) return receipt({page, offset: 0}, false);
    census = listing.total_count;
    for (let offset = page === cursor.page ? cursor.offset : 0; offset < listing.artifacts.length; offset++) {
      if (examined === maxOwners) return receipt({page, offset}, false);
      examined++;
      const metadata = listing.artifacts[offset];
      try {
        const prepared = await prepareCleanupClaim({reader, metadata, producer, cache, ...dependencies});
        const key = `${prepared.run}/${prepared.attempt}`;
        if (seen.has(key)) {
          const previous = seen.get(key), index = owners.indexOf(previous);
          if (index >= 0) owners.splice(index, 1);
          retained.push({artifact: metadata.id, reason: 'ambiguous-owner-artifacts'}); continue;
        }
        seen.set(key, prepared);
        if (prepared.eligible) owners.push(prepared);
        else retained.push({artifact: metadata.id, run: prepared.run, attempt: prepared.attempt, reason: prepared.reason});
      } catch {
        // Unproved artifact values and transport responses are never exported as personal diagnostics.
        retained.push({artifact: metadata?.id ?? null, reason: 'effect-owner-proof-unavailable-or-invalid'});
      }
    }
    if (listing.artifacts.length < 100 || page * 100 >= census) return receipt(null, true);
  }
  return receipt({page: cursor.page + maxPages, offset: 0}, false);
}
export async function applyOwnedHiveClaim({reader, producer, claimArtifactId, client = createEvidenceClient({
  bounds: {retries: 0, logBytes: 1048576, logDownloadBytes: 8388608}}), ...dependencies} = {}) {
  return applyCleanupClaim({reader, producer, claimArtifactId, client, ...dependencies});
}
function producerFromEnvironment(environment) {
  assert.equal(environment.GITHUB_REPOSITORY, repository);
  return {run: Number(environment.GITHUB_RUN_ID), attempt: Number(environment.GITHUB_RUN_ATTEMPT),
    protocolCommit: environment.GITHUB_SHA};
}
function outputDocument(directory, name, document) {
  fs.mkdirSync(directory, {recursive: true}); const file = path.join(directory, name);
  fs.writeFileSync(file, JSON.stringify(document, null, 2) + '\n', {flag: 'wx', mode: 0o600}); return file;
}
export function writeCleanupPreparation(result, directory) {
  const owners = result.owners.map(owner => {
    cleanupClaimName(owner.run, owner.attempt);
    const target = path.join(directory, `${owner.run}-${owner.attempt}`);
    return {...owner, claimPath: outputDocument(target, 'claim.json', owner.claim)};
  });
  return {...result, owners, progressPath: outputDocument(directory, 'progress.json', result.progress)};
}
export function writeCleanupOutcome(outcome, directory) {
  cleanupClaimName(outcome.sourceRun, outcome.sourceAttempt);
  const target = path.join(directory, `${outcome.sourceRun}-${outcome.sourceAttempt}`);
  return {...outcome, outcomePath: outputDocument(target, 'outcome.json', outcome)};
}
export function writeCleanupRecheck(result, directory) {
  cleanupClaimName(result.run, result.attempt); assert.equal(typeof result.eligible, 'boolean');
  const claimPath = result.eligible
    ? outputDocument(path.join(directory, `${result.run}-${result.attempt}`), 'claim.json', result.claim) : null;
  return {...result, schema: 'owned-hive-cleanup-preclaim/v1', claimPath};
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [phase, ...args] = process.argv.slice(2);
  assert.ok(['prepare', 'recheck', 'apply'].includes(phase), 'Usage: finalize-owned-hive-delivery-runs.mjs prepare|recheck|apply');
  const {githubReader} = await import('./admit-hive-original-delivery.mjs');
  const reader = githubReader({repository}), producer = producerFromEnvironment(process.env);
  const directory = path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), `hive-cleanup-${producer.run}-${producer.attempt}`);
  if (phase === 'recheck') {
    assert.equal(args.length, 0); const input = process.env.CLEANUP_CLAIM ?? '';
    assert.ok(Buffer.byteLength(input, 'utf8') <= 131072, 'bounded queued claim');
    const result = await revalidateCleanupClaim({reader, producer, claim: JSON.parse(input)});
    const published = writeCleanupRecheck(result, directory);
    assert.ok(process.env.GITHUB_OUTPUT, 'workflow step output is required');
    fs.appendFileSync(process.env.GITHUB_OUTPUT, `eligible=${published.eligible}\nclaim_path=${published.claimPath ?? ''}\n`);
    process.stdout.write(JSON.stringify(published, null, 2) + '\n');
  } else if (phase === 'apply') {
    assert.equal(args.length, 1); assert.match(args[0], /^[1-9]\d*$/u);
    const outcome = await applyOwnedHiveClaim({reader, producer, claimArtifactId: Number(args[0])});
    process.stdout.write(JSON.stringify(writeCleanupOutcome(outcome, directory), null, 2) + '\n');
    process.stderr.write(`Owned cleanup: state=${outcome.state}; owner=${outcome.sourceRun}/${outcome.sourceAttempt}; `
      + `closed=${outcome.cleanupReceipt?.closed ?? 'unknown'}; retained=${outcome.cleanupReceipt?.retained ?? 'unknown'}; `
      + `delivery-credit=false\n`);
    if (!['completed', 'not-started'].includes(outcome.state)) process.exitCode = 1;
  } else {
    const options = new Map();
    for (const arg of args) {
      assert.match(arg, /^--(?:cursor=[1-9]\d*:\d+|pages=[1-9]\d*|owners=[1-9]\d*)$/u);
      const [key, value] = arg.split('='); assert.equal(options.has(key), false); options.set(key, value);
    }
    const position = options.has('--cursor') ? options.get('--cursor').split(':').map(Number) : null;
    const trigger = process.env.GITHUB_EVENT_NAME === 'workflow_run'
      ? JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8')).workflow_run : null;
    const result = await prepareOwnedHiveRuns({reader, producer, trigger,
      cursor: position ? {page: position[0], offset: position[1]} : null,
      maxPages: Number(options.get('--pages') ?? 3), maxOwners: Number(options.get('--owners') ?? 3)});
    process.stdout.write(JSON.stringify(writeCleanupPreparation(result, directory), null, 2) + '\n');
  }
}
