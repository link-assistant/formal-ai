#!/usr/bin/env node
// Issue #1191 (requirement 11): the weekly status job reports open pull
// requests that nobody has touched for a while. Five pull requests sat open
// and conflicting for months (#887, #652, #646, #644, #643) without anything
// saying so. This is a report, not a gate: it writes the list to the step
// summary and annotates each pull request with a warning, then exits 0.
//
// Usage: report-stale-pull-requests.mjs [--days N] [--repo owner/name] [--json FILE]
import {execFileSync} from 'node:child_process';
import fs from 'node:fs';
import {pathToFileURL} from 'node:url';

const DAY_MS = 86_400_000;
const MAX_ANNOTATIONS = 20;

/** Open pull requests whose last update is at least `days` old, oldest first. */
export function stalePullRequests(pulls, {now = Date.now(), days = 30} = {}) {
  if (!Number.isSafeInteger(days) || days < 1) throw new Error('days must be a positive integer');
  return pulls
    .filter(pull => pull.state === undefined || pull.state === 'open')
    .map(pull => ({
      number: pull.number,
      title: String(pull.title ?? ''),
      branch: pull.head?.ref ?? '',
      draft: pull.draft === true,
      createdAt: pull.created_at,
      updatedAt: pull.updated_at,
      idleDays: Math.floor((now - Date.parse(pull.updated_at)) / DAY_MS),
    }))
    .filter(pull => Number.isFinite(pull.idleDays) && pull.idleDays >= days)
    .sort((a, b) => b.idleDays - a.idleDays || a.number - b.number);
}

const cell = text => text.replace(/[|\r\n]/gu, ' ').slice(0, 90);

/** Markdown for the step summary. */
export function renderReport(stale, {days, total, repo}) {
  const lines = [`### Stale pull requests (no update for ${days}+ days)`, ''];
  if (stale.length === 0) {
    lines.push(`None: all ${total} open pull requests were updated in the last ${days} days.`);
    return lines.join('\n') + '\n';
  }
  lines.push(`${stale.length} of ${total} open pull requests are stale. Merge, close, or comment why each stays open.`, '',
    '| PR | idle days | opened | draft | branch | title |', '| --- | ---: | --- | --- | --- | --- |');
  for (const pull of stale) {
    lines.push(`| [#${pull.number}](https://github.com/${repo}/pull/${pull.number}) | ${pull.idleDays} | ${pull.createdAt.slice(0, 10)} | ${pull.draft ? 'yes' : 'no'} | \`${cell(pull.branch)}\` | ${cell(pull.title)} |`);
  }
  return lines.join('\n') + '\n';
}

/** One `::warning` per stale pull request, capped so the run page stays readable. */
export function annotations(stale, {days}) {
  const shown = stale.slice(0, MAX_ANNOTATIONS).map(pull =>
    `::warning title=Stale pull request #${pull.number}::#${pull.number} has had no update for ${pull.idleDays} days (threshold ${days}): ${cell(pull.title)}`);
  if (stale.length > MAX_ANNOTATIONS) shown.push(`::warning title=Stale pull requests::${stale.length - MAX_ANNOTATIONS} more stale pull requests are listed in the step summary`);
  return shown;
}

export function parseArgs(args) {
  const options = {days: 30, repo: process.env.GITHUB_REPOSITORY || 'link-assistant/formal-ai', json: null};
  for (let index = 0; index < args.length; index += 2) {
    const [flag, value] = [args[index], args[index + 1]];
    if (value === undefined) throw new Error(`${flag} needs a value`);
    if (flag === '--days') options.days = Number(value);
    else if (flag === '--repo') options.repo = value;
    else if (flag === '--json') options.json = value;
    else throw new Error(`Unknown option ${flag}`);
  }
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u.test(options.repo)) throw new Error('Invalid --repo');
  return options;
}

/** Open pull requests through `gh`, projected to the fields the report reads (full objects exceed 1 MB per page). */
function fetchOpenPulls(repo) {
  const projection = '.[] | {number, title, state, draft, created_at, updated_at, head: {ref: .head.ref}}';
  const output = execFileSync('gh', ['api', '--paginate', `repos/${repo}/pulls?state=open&per_page=100`, '--jq', projection],
    {encoding: 'utf8', timeout: 120_000, maxBuffer: 16 * 1024 * 1024});
  return output.split('\n').filter(Boolean).map(line => JSON.parse(line));
}

export async function main(args = process.argv.slice(2), {fetchPulls = fetchOpenPulls, now = Date.now(), log = console.log, summary = process.env.GITHUB_STEP_SUMMARY} = {}) {
  const options = parseArgs(args);
  const pulls = await fetchPulls(options.repo);
  const stale = stalePullRequests(pulls, {now, days: options.days});
  const report = renderReport(stale, {days: options.days, total: pulls.length, repo: options.repo});
  for (const line of annotations(stale, {days: options.days})) log(line);
  log(report);
  if (summary) fs.appendFileSync(summary, report);
  if (options.json) fs.writeFileSync(options.json, JSON.stringify({days: options.days, total: pulls.length, stale}, null, 2) + '\n');
  return stale;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main().catch(error => {
  console.error(error.message); process.exitCode = 1;
});
