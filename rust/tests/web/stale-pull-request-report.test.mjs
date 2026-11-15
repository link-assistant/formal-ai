// Issue #1191: requirement 11 (stale pull-request report in the weekly status
// job) and requirement 7 (required status checks on main, a maintainer
// action that the same weekly job reports on main). These tests use fixtures,
// so pull requests never call the GitHub API.

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { annotations, main as reportMain, parseArgs, renderReport, stalePullRequests } from '../../../scripts/report-stale-pull-requests.mjs';
import { REQUIRED_CHECKS, main as checkMain, missingChecks, remedy } from '../../../scripts/check-required-status-checks.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const read = relative => readFileSync(join(ROOT, relative), 'utf8');
const NOW = Date.parse('2026-11-15T00:00:00Z');

// The five pull requests the issue named, as the API reported them on 2026-11-15.
const NAMED = [
  [887, 'Add deterministic anticipatory dreaming', 'issue-705-5bb14827217a', '2026-08-01T16:46:59Z'],
  [652, 'Plan the critical missing features as tracked sub-issues; refresh vision and roadmap', 'issue-651-0f33c6af0d5a', '2026-07-12T20:29:00Z'],
  [646, 'Fix sidebar splitter affordance and scrolling clarity', 'issue-447-2d79997f934e', '2026-07-10T20:51:36Z'],
  [644, 'Add experimental formalization model fallback', 'issue-483-69f2ba125406', '2026-07-09T00:41:38Z'],
  [643, 'Add polished multi-framework UI skins and color themes', 'issue-557-798448f62fb9', '2026-07-19T23:50:08Z'],
].map(([number, title, ref, updated]) => ({ number, title, state: 'open', draft: false, created_at: updated, updated_at: updated, head: { ref } }));
const FRESH = { number: 1573, title: 'Keep JS-first CI fast | close the gap', state: 'open', draft: true, created_at: '2026-11-10T00:00:00Z', updated_at: '2026-11-14T00:00:00Z', head: { ref: 'issue-1191-a377d78ccc05' } };

test('the five pull requests the issue named are reported, oldest first; a fresh one is not', () => {
  const stale = stalePullRequests([FRESH, ...NAMED], { now: NOW, days: 30 });
  assert.deepEqual(stale.map(pull => pull.number), [644, 646, 652, 643, 887]);
  assert.equal(stale[0].idleDays, 128);
  assert.deepEqual(stalePullRequests([FRESH], { now: NOW, days: 30 }), []);
  assert.deepEqual(stalePullRequests([{ ...NAMED[0], state: 'closed' }], { now: NOW, days: 30 }), []);
  assert.throws(() => stalePullRequests([], { days: 0 }), /positive integer/u);
});

test('the report is a markdown table and one warning per pull request, capped', () => {
  const stale = stalePullRequests(NAMED, { now: NOW, days: 30 });
  const report = renderReport(stale, { days: 30, total: 6, repo: 'link-assistant/formal-ai' });
  assert.match(report, /5 of 6 open pull requests are stale/u);
  assert.match(report, /\| \[#887\]\(https:\/\/github\.com\/link-assistant\/formal-ai\/pull\/887\) \| 105 \| 2026-08-01 \| no \| `issue-705-5bb14827217a` \|/u);
  assert.match(renderReport([], { days: 30, total: 1, repo: 'r/r' }), /None: all 1 open pull requests/u);
  assert.equal(annotations(stale, { days: 30 }).length, 5);
  assert.match(annotations(stale, { days: 30 })[0], /^::warning title=Stale pull request #644::/u);
  const many = Array.from({ length: 25 }, (_, index) => ({ ...stale[0], number: index }));
  assert.equal(annotations(many, { days: 30 }).length, 21);
  const table = renderReport(stalePullRequests([{ ...FRESH, updated_at: '2026-01-01T00:00:00Z' }], { now: NOW, days: 30 }), { days: 30, total: 1, repo: 'r/r' });
  assert.doesNotMatch(table.split('\n').at(-2), /JS-first CI fast \| close/u, 'a pipe in a title cannot break the table');
});

test('the report never fails the job and rejects bad options', async () => {
  const lines = [];
  const stale = await reportMain(['--days', '30', '--repo', 'link-assistant/formal-ai'], { fetchPulls: async () => [FRESH, ...NAMED], now: NOW, log: line => lines.push(line), summary: null });
  assert.equal(stale.length, 5);
  assert.equal(lines.filter(line => line.startsWith('::warning')).length, 5);
  assert.throws(() => parseArgs(['--days']), /needs a value/u);
  assert.throws(() => parseArgs(['--repo', 'bad repo']), /Invalid --repo/u);
  assert.throws(() => parseArgs(['--colour', 'x']), /Unknown option/u);
});

test('the weekly status job runs the report and the required-check check on main', () => {
  const workflow = read('.github/workflows/job-headroom.yml');
  assert.match(workflow, /cron: '43 4 \* \* 1'/u, 'weekly');
  assert.match(workflow, /pull-requests: read/u);
  assert.match(workflow, /run: node scripts\/report-stale-pull-requests\.mjs --days 30/u);
  assert.match(workflow, /run: node scripts\/check-required-status-checks\.mjs --branch main/u);
  assert.doesNotMatch(workflow, /^\s+pull_request:/mu, 'pull requests run only these fixture tests');
});

// The ruleset as it stood on 2026-11-15 (gh api repos/link-assistant/formal-ai/rules/branches/main).
const TODAY = [{ type: 'deletion' }, { type: 'non_fast_forward' }];
const REQUIRED = [...TODAY, { type: 'required_status_checks', parameters: { required_status_checks: REQUIRED_CHECKS.map(context => ({ context })) } }];

test('main fails with the exact command until the ruleset requires the three checks', async () => {
  assert.deepEqual(missingChecks(TODAY), REQUIRED_CHECKS);
  assert.deepEqual(missingChecks(REQUIRED), []);
  assert.deepEqual(missingChecks([...TODAY, { type: 'required_status_checks', parameters: { required_status_checks: [{ context: 'Pipeline Status' }] } }]),
    REQUIRED_CHECKS.filter(name => name !== 'Pipeline Status'));
  const errors = [];
  assert.equal(await checkMain(['--branch', 'main'], { getRules: async () => TODAY, log: () => {}, error: line => errors.push(line) }), 1);
  assert.match(errors[0], /^::error title=Required status checks missing on main::/u);
  const message = remedy(REQUIRED_CHECKS, { repo: 'link-assistant/formal-ai', branch: 'main' });
  assert.match(message, /gh api -X POST repos\/link-assistant\/formal-ai\/rulesets --input -/u);
  assert.match(message, /docs-only/u);
  assert.match(message, /external-benchmarks\.yml/u);
  const body = JSON.parse(message.split('\n').at(-2));
  assert.deepEqual(body.rules[0].parameters.required_status_checks.map(check => check.context), REQUIRED_CHECKS);
  assert.equal(await checkMain([], { getRules: async () => REQUIRED, log: () => {}, error: () => {} }), 0);
});

test('every required check name is a job the pipeline really reports', () => {
  const release = read('.github/workflows/release.yml');
  assert.match(release, /\n {4}name: Fast Linux source and requirement checks\n/u);
  assert.match(release, /\n {4}name: Native type and lint check\n/u);
  assert.match(read('.github/workflows/native-type-check.yml'), /\n {4}name: Rust compile inputs changed\n/u);
  assert.match(release, /\n {4}name: Pipeline Status\n/u);
});
