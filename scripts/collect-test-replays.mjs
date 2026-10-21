#!/usr/bin/env node
// Bounded source-qualified evidence, never raw session uploads or invented replay events.
import {pathToFileURL} from 'node:url';
import {isFloat, parseJson, toCompactJson} from '../js/server/json.mjs';
import originalInputs from '../data/benchmarks/hive-original-delivery-inputs.json' with {type: 'json'};
import {validateBounds, runGh, redact, diagnostics, deduplicate} from './collect-ci-failures.mjs';
import {digest} from './lib/generated-task-custody.mjs';
import {requireExactProgramOutput, requireHarnessDelivery, requireReadyPullRequest,
  requireProgramMember} from './lib/hive-delivery-evidence.mjs';

export function createEvidenceClient({bounds: input = {}, aggregateBytes = 268435456,
  maxRequests = 1000, transport = runGh} = {}) {
  const bounds = validateBounds(input);
  if (!Number.isSafeInteger(aggregateBytes) || aggregateBytes < 1 || aggregateBytes > 268435456) throw new Error('Invalid aggregateBytes');
  if (!Number.isSafeInteger(maxRequests) || maxRequests < 1 || maxRequests > 1000) throw new Error('Invalid maxRequests');
  const attempts = []; let bytes = 0;
  async function request(endpoint, {log = false, full = false, mutation = null} = {}) {
    if (!/^repos\/[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+\//u.test(endpoint) || /[\r\n]/u.test(endpoint)) throw new Error('Invalid API endpoint');
    const count = mutation ? 1 : bounds.retries + 1;
    let last;
    for (let attempt = 0; attempt < count; attempt++) {
      if (attempts.length >= maxRequests || bytes >= aggregateBytes) throw new Error('Evidence collection budget exhausted');
      const remaining = aggregateBytes - bytes;
      const transfer = Math.min(log ? bounds.logDownloadBytes : 1048576, remaining);
      const args = ['api', endpoint, ...(mutation ?? [])];
      last = await transport(args, {timeoutMs: bounds.timeoutMs,
        maxBytes: Math.min(log && !full ? bounds.logBytes : transfer, transfer), maxDownloadBytes: transfer});
      const received = last.bytesReceived ?? Buffer.byteLength(last.text ?? '');
      if (!Number.isSafeInteger(received) || received < 0 || received > transfer) throw new Error('Transport exceeded declared byte bound');
      bytes += received;
      attempts.push({endpoint, attempt, mutation: Boolean(mutation), ok: last.ok === true,
        bytes: received, timedOut: last.timedOut === true, truncated: last.truncated === true,
        error: redact(last.error ?? '').slice(0, 500)});
      if (last.ok || mutation) return last;
    }
    return last;
  }
  return {bounds, attempts, request,
    async json(endpoint) {
      const result = await request(endpoint);
      if (!result.ok || result.truncated || result.downloadTruncated) throw new Error(`Unavailable complete API metadata: ${redact(endpoint)}`);
      return JSON.parse(result.text);
    }, transferredBytes: () => bytes};
}
const privateMetadataKeys = new Set(['user', 'actor', 'author', 'committer', 'account', 'person', 'profile',
  'email', 'emailaddress', 'login', 'username', 'avatarurl', 'accountid', 'userid', 'actorid',
  'authorid', 'committerid', 'personid', 'accountname', 'actorname', 'personname', 'displayname',
  'token', 'authorization', 'password', 'secret', 'apikey', 'accesstoken', 'refreshtoken',
  'idtoken', 'clientsecret', 'cookie', 'setcookie', 'privatekey', 'accesskey', 'sessiontoken', 'bearertoken']);
const personalIdentifiers = new Set(['email', 'emailaddress', 'login', 'username',
  'accountid', 'userid', 'actorid', 'personid', 'displayname', 'fullname']);
export function sanitizeEvidence(value, context = '') {
  if (isFloat(value)) return value;
  if (typeof value === 'string') {
    if (/^\s*(?:\{|\[|")/u.test(value)) {
      try { return toCompactJson(sanitizeEvidence(parseJson(value), context)); } catch { /* Non-JSON below. */ }
    }
    return redact(value).replace(/^(?:author|committer|username|authenticated user):.*$/gimu, '[account metadata removed]');
  }
  if (Array.isArray(value)) return value.map(item => sanitizeEvidence(item, context));
  if (value && typeof value === 'object') {
    const entries = Object.entries(value).map(([key, item]) =>
      ({key, item, normalized: key.replace(/[_-]/gu, '').toLowerCase()}));
    if (entries.some(row => row.normalized === 'login')) return '[account metadata removed]';
    const repositoryContext = ['repo', 'repository'].includes(context);
    const personal = entries.some(row => personalIdentifiers.has(row.normalized)
      && !(repositoryContext && row.normalized === 'fullname'));
    return Object.fromEntries(entries.filter(row => !privateMetadataKeys.has(row.normalized)
      && (row.normalized !== 'fullname' || repositoryContext)
      && !(personal && ['id', 'name'].includes(row.normalized)))
      .map(row => [row.key, sanitizeEvidence(row.item, row.normalized)]));
  }
  return value;
}
export function replayOriginalDelivery(fixture) {
  if (fixture?.schema !== 'original-hive-delivery-replay/v1' || fixture.complete !== true) throw new Error('Incomplete structured replay');
  if (fixture.language !== originalInputs.clients[fixture.tool] || fixture.expectedStdout !== originalInputs.expectedStdout) throw new Error('Fixture changed original task oracle');
  if (!/^[a-f0-9]{40}$/u.test(fixture.provenance?.sourceHead ?? '')
    || !/^[a-f0-9]{64}$/u.test(fixture.provenance?.sourceBinarySha256 ?? '')
    || !/^[a-f0-9]{64}$/u.test(fixture.provenance?.originalPromptSha256 ?? '')) throw new Error('Missing replay source identity');
  const selected = requireHarnessDelivery(fixture.harness, fixture.tool);
  requireReadyPullRequest(fixture.pullRequest, selected.repository, selected.number);
  requireProgramMember(fixture.files, fixture.language);
  requireExactProgramOutput(fixture.independentProgram, originalInputs.expectedStdout);
  return {passed: true, oracle: 'unchanged-original-hive-delivery', sourceHead: fixture.provenance.sourceHead,
    sourceAuthentication: 'requires-external-sealed-artifact-verification', autonomousAuthorshipCredit: 0};
}
/** The caller must authenticate the sealed manifest to its hosted run before supplying expected. */
export function collectReplayFixture({manifest, documents, expected}) {
  if (manifest?.schema !== 'sealed-public-e2e-replay/v1' || manifest.complete !== true
    || manifest.workload !== 'public-generated-hello-world') throw new Error('Unproved public replay source');
  for (const key of ['run', 'job', 'sourceHead', 'sourceBinarySha256', 'originalPromptSha256']) {
    if (expected?.[key] === undefined || manifest[key] !== expected[key]) throw new Error(`Replay source mismatch: ${key}`);
  }
  if (![manifest.run, manifest.job].every(value => Number.isSafeInteger(value) && value > 0)
    || !['success', 'failure', 'cancelled'].includes(manifest.outcome)) throw new Error('Invalid replay observation');
  if (!/^[a-f0-9]{40}$/u.test(manifest.sourceHead ?? '')
    || !/^[a-f0-9]{64}$/u.test(manifest.sourceBinarySha256 ?? '')
    || !/^[a-f0-9]{64}$/u.test(manifest.originalPromptSha256 ?? '')
    || !Object.hasOwn(originalInputs.clients, manifest.tool)) throw new Error('Invalid replay source identity');
  const names = ['harness', 'pullRequest', 'files', 'independentProgram', 'toolEvents'];
  if (!Array.isArray(manifest.members) || manifest.members.length !== names.length
    || Object.keys(documents ?? {}).length !== names.length) throw new Error('Incomplete replay document inventory');
  const values = {}; let total = 0;
  for (const name of names) {
    const rows = manifest.members.filter(row => row.name === name), text = documents[name];
    if (rows.length !== 1 || typeof text !== 'string') throw new Error('Missing unique replay document');
    const bytes = Buffer.byteLength(text); total += bytes;
    if (bytes > 8388608 || total > 33554432 || rows[0].bytes !== bytes
      || rows[0].sha256 !== digest(text)) throw new Error('Replay document bytes differ from sealed source');
    values[name] = JSON.parse(text);
  }
  if (!Array.isArray(values.toolEvents) || values.toolEvents.length > 10000) throw new Error('Invalid finite tool event inventory');
  const fixture = {schema: 'original-hive-delivery-replay/v1', complete: true,
    tool: manifest.tool, language: originalInputs.clients[manifest.tool],
    expectedStdout: originalInputs.expectedStdout, provenance: {...expected, outcome: manifest.outcome,
      sourceMembers: manifest.members, externalSourceAuthenticationRequired: true}, ...values};
  const evaluate = value => {
    try { replayOriginalDelivery(value); return {passed: true}; }
    catch (error) { return {passed: false, errorFingerprint: digest(error.message)}; }
  };
  const sanitized = sanitizeEvidence(fixture), originalOracle = evaluate(fixture), sanitizedOracle = evaluate(sanitized);
  if (JSON.stringify(originalOracle) !== JSON.stringify(sanitizedOracle)) throw new Error('Sanitization changed original delivery oracle');
  return {...sanitized, observedOutcome: manifest.outcome, originalOracle: sanitizedOracle,
    outcomeConsistentWithOriginalOracle: (manifest.outcome === 'success') === sanitizedOracle.passed,
    toolTraceSemanticEquivalence: 'unverified', rawPrivateDataRetained: false,
    scope: 'source-qualified sanitized delivery-oracle replay; not whole-client planning acceptance'};
}
export async function collectReplayEvidence({repository = 'link-assistant/formal-ai',
  workflow = 'hive-original-delivery.yml', bounds = {}, aggregateBytes, maxRequests, transport} = {}) {
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u.test(repository)
    || !/^[A-Za-z0-9_.-]+\.yml$/u.test(workflow)) throw new Error('Invalid evidence owner');
  const client = createEvidenceClient({bounds, aggregateBytes, maxRequests, transport});
  const result = {schema: 'original-e2e-evidence-catalog/v1', repository, workflow,
    runs: [], occurrences: [], unavailable: [], rawLogsPersisted: false,
    scope: {runLimit: client.bounds.runs, pageLimit: client.bounds.pages, jobsPerRun: client.bounds.jobs}};
  const seen = new Set();
  for (let page = 1; page <= client.bounds.pages && seen.size < client.bounds.runs; page++) {
    let listing;
    try { listing = await client.json(`repos/${repository}/actions/workflows/${workflow}/runs?per_page=100&page=${page}`); }
    catch (error) { result.unavailable.push({page, error: redact(error.message)}); break; }
    result.scope.availableRuns = listing.total_count ?? null;
    result.scope.pagesObserved = page;
    for (const run of listing.workflow_runs ?? []) {
      if (seen.size >= client.bounds.runs) break;
      if (seen.has(run.id)) continue;
      seen.add(run.id);
      const record = {id: run.id, url: run.html_url, head: run.head_sha, status: run.status,
        conclusion: run.conclusion, event: run.event, originalPullRequests: (run.pull_requests ?? []).map(row => row.number), jobs: []};
      result.runs.push(record);
      try {
        const jobs = await client.json(`repos/${repository}/actions/runs/${run.id}/jobs?per_page=100`);
        record.jobsTruncated = jobs.total_count > Math.min(100, client.bounds.jobs);
        record.jobsMetadataIncomplete = (jobs.jobs ?? []).length < Math.min(100, jobs.total_count);
        for (const job of (jobs.jobs ?? []).slice(0, client.bounds.jobs)) {
          const provenance = {repository, run: run.id, head: run.head_sha, job: job.id, name: job.name, url: job.html_url};
          const entry = {...provenance, status: job.status, conclusion: job.conclusion,
            steps: (job.steps ?? []).map(step => ({name: redact(step.name), status: step.status, conclusion: step.conclusion}))};
          record.jobs.push(entry);
          if (job.status !== 'completed' || job.conclusion === 'skipped') {
            entry.logsUnavailable = true; entry.reason = 'job-not-complete-or-skipped'; continue;
          }
          const log = await client.request(`repos/${repository}/actions/jobs/${job.id}/logs`, {log: true});
          entry.logsUnavailable = !log.text;
          entry.retainedTruncated = log.truncated === true;
          entry.transferTruncated = log.downloadTruncated === true;
          entry.streamComplete = log.streamComplete === true;
          entry.transferComplete = log.ok === true && log.downloadTruncated !== true;
          entry.timedOut = log.timedOut === true;
          entry.cleanupComplete = log.cleanupComplete === true;
          entry.tailComplete = log.tailComplete === true;
          entry.transferredBytes = log.bytesReceived ?? Buffer.byteLength(log.text ?? '');
          entry.retainedTextSha256 = digest(log.text ?? '');
          entry.diagnostics = diagnostics(log.text ?? '').map(({diagnostic, ...item}) => ({...item,
            codes: [...new Set(diagnostic.match(/\b(?:e(?:inval|acces|noent|nospc|pipe|connreset|timedout|perm|[0-9]{4})|err_[a-z0-9_]+|http [0-9]{3})\b/gu) ?? [])],
            argumentNames: [...new Set(diagnostic.match(/--[a-z][a-z0-9-]*/gu) ?? [])],
            freeformMessagePersisted: false}));
          entry.structuredReplayAvailable = false;
          entry.missingReplayFields = ['separate stdout/stderr', 'complete tool request/result events',
            'original prompt digest', 'same-run binary digest', 'independent program receipt'];
          result.occurrences.push({...provenance, conclusion: job.conclusion,
            logsUnavailable: entry.logsUnavailable, diagnostics: entry.diagnostics});
        }
      } catch (error) { record.unavailable = redact(error.message); }
    }
    if ((listing.workflow_runs ?? []).length < 100) break;
  }
  result.groups = deduplicate(result.occurrences);
  result.scope.runsTruncated = result.scope.availableRuns === null
    || result.scope.availableRuns === undefined || result.scope.availableRuns > seen.size;
  result.requests = client.attempts;
  result.transferredBytes = client.transferredBytes();
  return sanitizeEvidence(result);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const repository = process.argv[2] ?? 'link-assistant/formal-ai';
  collectReplayEvidence({repository}).then(result => process.stdout.write(JSON.stringify(result, null, 2) + '\n'))
    .catch(error => { process.stderr.write(redact(error.message) + '\n'); process.exitCode = 1; });
}
