#!/usr/bin/env node
// Read-only, bounded GitHub Actions diagnostics. Never persist raw job logs.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';

const defaults = { runs: 20, pages: 2, jobs: 8, logBytes: 65536, logDownloadBytes: 2097152, timeoutMs: 20000, retries: 1 };
const maxima = { runs: 100, pages: 5, jobs: 30, logBytes: 1048576, logDownloadBytes: 8388608, timeoutMs: 60000, retries: 2 };
export function validateBounds(options = {}) {
  if (options === null || typeof options !== 'object' || Array.isArray(options) || Object.keys(options).some(key => !Object.hasOwn(defaults, key))) throw new Error('Unknown bounds');
  const result = { ...defaults, ...options };
  for (const key of Object.keys(defaults)) {
    if (!Number.isSafeInteger(result[key]) || result[key] < (key === 'retries' ? 0 : 1) || result[key] > maxima[key]) {
      throw new Error(`Invalid ${key}: finite integer within ${key === 'retries' ? 0 : 1}..${maxima[key]} required`);
    }
  }
  return result;
}
export function redact(value) {
  return String(value)
    .replace(/\b(?:gh[pousr]_[A-Za-z0-9_]+|github_pat_[A-Za-z0-9_]+|AKIA[A-Z0-9]{16})\b/g, '[credential]')
    .replace(/https?:\/\/[^\s?]+\?[^\s]*\b(?:sig|token|credential|key)=[^\s]+/gi, '[signed-url]')
    .replace(/\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b/g, '[credential]')
    .replace(/\b(Bearer|Basic)\s+\S+/gi, '$1 [credential]')
    .replace(/\b(token|password|secret|authorization|api[_-]?key)["']?\s*[=:]\s*["']?[^\s"']+/gi, '$1=[credential]')
    .replace(/https?:\/\/[^\s/@]+:[^\s/@]+@/gi, 'https://[credential]@')
    .replace(/[\w.+-]+@[\w.-]+\.[A-Za-z]{2,}/g, '[email]')
    .replace(/\b\d{12}\b/g, '[account]')
    .replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g, '[ip]')
    .replace(/\/(?:Users|home)\/[^/\s]+/g, '/home/[user]');
}
export function normalize(value) {
  return redact(value).toLowerCase()
    .replace(/\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/g, '<uuid>')
    .replace(/\b\d{4}-\d\d-\d\d[t ][\d:.]+z?\b/g, '<time>')
    .replace(/\b[0-9a-f]{40,64}\b/g, '<sha>')
    .replace(/\/(?:private\/)?tmp\/[^\s:'"()]+/g, '/tmp/<path>')
    .replace(/\b(?:duration|elapsed)[=: ]+\d+(?:\.\d+)?(?:ms|s)?\b/g, 'duration=<time>')
    .replace(/\s+/g, ' ').trim();
}
const categories = [
  ['github-integration-permission', /resource not accessible by integration|failed to fetch authenticated github username/iu],
  ['invalid-action-arguments', /invalid (?:action |tool )?arguments|unknown (?:argument|field)|unexpected (?:argument|keyword)|missing required (?:argument|field)|unknown action/iu],
  ['schema-binding', /schema.{0,70}(?:bind|mismatch|invalid)|(?:bind|mismatch).{0,70}schema/iu],
  ['no-progress', /repetit|no progress|same action|loop detected/iu],
  ['refspec', /refspec|couldn.t find remote ref|not a valid object name/iu],
  ['no-delivered-changes', /no (?:delivered )?changes|finished without changes|nothing to commit|empty (?:diff|delivery)/iu],
  ['log-unwritable', /(?:log|transcript).{0,80}(?:permission denied|unwritable|eacces|enospc)|(?:eacces|enospc).{0,80}(?:log|transcript)/iu],
];
export function diagnostics(log) {
  const found = new Map();
  for (const line of String(log).split(/\r?\n/u)) {
    const clean = line.replace(/^\d{4}-\d\d-\d\dT[\d:.]+Z\s*/u, '').replace(/\x1b\[[0-9;]*m/gu, '');
    // Agent payloads, echoed scripts and configuration are context, not observed failures.
    if (/^[{\[]|^"|^if\b|^echo\b|^curl\b|^if-no-files-found:|\b0 failed\b|^failed distributed compilations\s+0$/iu.test(clean.trim())) continue;
    const category = categories.find(([, pattern]) => pattern.test(clean))?.[0];
    if (!category && !/(?:##\[error\]|\b[a-z]*error\b|\b[a-z]*exception\b|\bERR_[A-Z0-9_]+\b|\bfatal\b|\bfailed\b|\bpanic\b)/iu.test(clean)) continue;
    const diagnostic = normalize(clean).slice(0, 1000);
    if (!diagnostic) continue;
    const fingerprint = createHash('sha256').update(`${category ?? 'other'}\n${diagnostic}`).digest('hex').slice(0, 24);
    if (!found.has(fingerprint) && found.size >= 40) {
      const generic = [...found].find(([, item]) => item.category === 'other');
      if (!category && !generic) continue;
      found.delete(generic?.[0] ?? found.keys().next().value);
    }
    found.set(fingerprint, { fingerprint, category: category ?? 'other', diagnostic });
  }
  return [...found.values()];
}
/** Observed receipts, separate from diagnostic candidates and the job conclusion. */
export function observedOutcome(log) {
  const lines = String(log).split(/\r?\n/u).map(line => line
    .replace(/^\d{4}-\d\d-\d\dT[\d:.]+Z\s*/u, '').replace(/\x1b\[[0-9;]*m/gu, ''))
    .filter(line => !/^[{\[]|^"/u.test(line.trim()));
  const denial = lines.findIndex(line => /failed to fetch authenticated github username|resource not accessible by integration/iu.test(line));
  const published = lines.findIndex((line, index) => index > denial && /Log URL: https:\/\/github\.com\/[^\s]+\/blob\//u.test(line));
  return {
    sourceDelivery: lines.some(line => /(?:^|\s)Finished without changes\s*$/iu.test(line)) ? 'no-changes-observed' : 'unknown',
    logPublication: denial >= 0 && published > denial ? 'recovered-after-permission-denial' : 'unknown',
  };
}
export function deduplicate(occurrences) {
  const groups = new Map();
  for (const { diagnostics: items, ...occurrence } of occurrences) {
    for (const item of items) {
      if (!groups.has(item.fingerprint)) groups.set(item.fingerprint, { ...item, occurrences: [] });
      groups.get(item.fingerprint).occurrences.push(occurrence);
    }
  }
  return [...groups.values()].sort((a, b) => b.occurrences.length - a.occurrences.length || a.fingerprint.localeCompare(b.fingerprint));
}
export function runGh(args, { timeoutMs, maxBytes, maxDownloadBytes = maxBytes }) {
  return new Promise(resolve => {
    const child = spawn('gh', args, { stdio: ['ignore', 'pipe', 'pipe'] });
    const firstLimit = Math.ceil(maxBytes / 2), tailLimit = Math.floor(maxBytes / 2);
    let first = Buffer.alloc(0), tail = Buffer.alloc(0), bytes = 0, stderr = '';
    let downloadTruncated = false, timedOut = false;
    const timer = setTimeout(() => { timedOut = true; child.kill('SIGKILL'); }, timeoutMs);
    child.stdout.on('data', chunk => {
      const accepted = chunk.subarray(0, Math.max(0, maxDownloadBytes - bytes));
      const needed = Math.max(0, firstLimit - first.length);
      if (needed) first = Buffer.concat([first, accepted.subarray(0, needed)]);
      const afterFirst = accepted.subarray(Math.min(needed, accepted.length));
      tail = Buffer.concat([tail, afterFirst]).subarray(-tailLimit);
      bytes += chunk.length;
      if (bytes > maxDownloadBytes) { downloadTruncated = true; child.kill('SIGKILL'); }
    });
    child.stderr.on('data', chunk => { stderr = (stderr + chunk.toString()).slice(0, 2048); });
    child.on('error', error => { clearTimeout(timer); resolve({ ok: false, text: '', error: redact(error.message), truncated: false, downloadTruncated, timedOut, tailComplete: false }); });
    child.on('close', code => {
      clearTimeout(timer);
      const truncated = bytes > maxBytes;
      const text = first.toString('utf8') + (truncated ? '\n[bounded middle omitted]\n' : '') + tail.toString('utf8');
      resolve({ ok: code === 0, text, error: redact(stderr), truncated, downloadTruncated, timedOut,
        bytesReceived: Math.min(bytes, maxDownloadBytes), tailComplete: code === 0 && !downloadTruncated });
    });
  });
}
export async function collect({ conclusion = null, repositories = ['link-assistant/formal-ai', 'link-assistant/hive-mind'], bounds: input = {}, transport = runGh } = {}) {
  const bounds = validateBounds(input); // Validate every bound before any IO.
  if (!Array.isArray(repositories) || !repositories.length || repositories.length > 5 || repositories.some(repo => !/^[\w.-]+\/[\w.-]+$/u.test(repo))) throw new Error('1..5 owner/repository names required');
  if (conclusion !== null && !['failure', 'cancelled', 'skipped'].includes(conclusion)) throw new Error('Invalid conclusion filter');
  const report = { conclusionFilter: conclusion, version: 1, collectedAt: new Date().toISOString(), bounds, repositories, failures: [], cancelled: [], skipped: [], other: [], unavailable: [], coverage: [], groups: [] };
  const occurrences = [];
  async function request(endpoint, log = false) {
    let result;
    for (let attempt = 0; attempt <= bounds.retries; attempt++) {
      result = await transport(['api', endpoint], { timeoutMs: bounds.timeoutMs, maxBytes: log ? bounds.logBytes : 2097152, maxDownloadBytes: log ? bounds.logDownloadBytes : 2097152 });
      if (result.ok || result.truncated) return { ...result, attempts: attempt + 1 };
      // Retry transient transport/server errors only, never authentication or missing logs.
      if (!result.timedOut && !/\b(?:429|50[0234])\b|connection reset|temporar/iu.test(result.error ?? '')) return { ...result, attempts: attempt + 1 };
    }
    return { ...result, attempts: bounds.retries + 1 };
  }
  async function json(endpoint) {
    const result = await request(endpoint);
    if (!result.ok || result.truncated) { report.unavailable.push({ endpoint, reason: result.timedOut ? 'timeout' : result.truncated ? 'response-too-large' : 'request-failed', attempts: result.attempts }); return null; }
    try { return JSON.parse(result.text); } catch { report.unavailable.push({ endpoint, reason: 'invalid-json' }); return null; }
  }
  for (const repository of repositories) {
    let seen = 0;
    for (let page = 1; page <= bounds.pages && seen < bounds.runs; page++) {
      const data = await json(`repos/${repository}/actions/runs?per_page=${Math.min(100, bounds.runs)}&page=${page}${conclusion ? '&status=' + conclusion : ''}`);
      if (!data) break;
      const runs = data.workflow_runs ?? [];
      for (const run of runs.slice(0, bounds.runs - seen)) {
        seen++;
        const metadata = { repository, runId: run.id, runUrl: run.html_url, headSha: run.head_sha, headBranch: redact(run.head_branch ?? ''), workflow: redact(run.name ?? ''), conclusion: run.conclusion, status: run.status };
        if (run.conclusion !== 'failure') { report[run.conclusion === 'cancelled' ? 'cancelled' : run.conclusion === 'skipped' ? 'skipped' : 'other'].push(metadata); continue; }
        const failure = { ...metadata, jobs: [], jobsTruncated: false };
        report.failures.push(failure);
        const jobsData = await json(`repos/${repository}/actions/runs/${run.id}/jobs?per_page=${bounds.jobs}&page=1`);
        if (!jobsData) { failure.jobsUnavailable = true; continue; }
        failure.jobsTruncated = (jobsData.total_count ?? 0) > bounds.jobs;
        for (const job of (jobsData.jobs ?? []).slice(0, bounds.jobs)) {
          const item = { ...metadata, jobId: job.id, jobUrl: job.html_url, jobName: redact(job.name ?? ''), jobConclusion: job.conclusion };
          if (job.conclusion !== 'failure') { failure.jobs.push(item); continue; }
          const result = await request(`repos/${repository}/actions/jobs/${job.id}/logs`, true);
          item.logsUnavailable = !result.ok && !result.truncated;
          item.logsTruncated = !!result.truncated;
          item.logsDownloadTruncated = !!result.downloadTruncated;
          item.logsTailComplete = result.tailComplete === true;
          item.logBytesReceived = result.bytesReceived ?? null;
          item.logsTimedOut = !!result.timedOut;
          item.logAttempts = result.attempts;
          item.diagnostics = diagnostics(result.text ?? '');
          item.observedOutcome = observedOutcome(result.text ?? '');
          failure.jobs.push(item);
          occurrences.push(item);
        }
      }
      if (runs.length < Math.min(100, bounds.runs)) break;
    }
    report.coverage.push({ repository, runsExamined: seen, bounded: seen >= bounds.runs });
  }
  report.groups = deduplicate(occurrences);
  return report;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const options = {}; const repositories = []; let conclusion = null;
    for (let i = 2; i < process.argv.length; i += 2) {
      const name = process.argv[i].replace(/^--/u, ''); const value = process.argv[i + 1];
      if (name === 'conclusion') conclusion = value;
      else if (name === 'repo') repositories.push(value);
      else if (Object.hasOwn(defaults, name)) options[name] = Number(value);
      else throw new Error(`Unknown option ${name}`);
    }
    console.log(JSON.stringify(await collect({ conclusion, bounds: options, ...(repositories.length ? { repositories } : {}) }), null, 2));
  } catch (error) { console.error(redact(error.message)); process.exitCode = 1; }
}
