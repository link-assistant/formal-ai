#!/usr/bin/env node
// Issue #1191 (requirement 7): main must require the JavaScript stage, the Rust
// stage and `Pipeline Status` before a merge. Changing the `main` ruleset is a
// maintainer action. An admin must also decide on bypass actors, because
// external-benchmarks.yml pushes its weekly ledger straight to main. So this
// check does not change anything. On main it fails with the exact command to
// run; pull requests only run its fixture test
// (rust/tests/web/stale-pull-request-report.test.mjs).
//
// Usage: check-required-status-checks.mjs [--repo owner/name] [--branch main]
import {pathToFileURL} from 'node:url';

/** The check-run names the ruleset must require, in the order the pipeline reports them. */
export const REQUIRED_CHECKS = Object.freeze([
  'Fast Linux source and requirement checks',
  'Native type and lint check / Rust compile inputs changed',
  'Pipeline Status',
]);

/** Contexts that the active `required_status_checks` rules on the branch require. */
export function requiredContexts(rules) {
  return new Set(rules
    .filter(rule => rule.type === 'required_status_checks')
    .flatMap(rule => rule.parameters?.required_status_checks ?? [])
    .map(check => check.context));
}

export function missingChecks(rules, required = REQUIRED_CHECKS) {
  const present = requiredContexts(rules);
  return required.filter(name => !present.has(name));
}

/** The message a maintainer acts on; printed as an `::error` on main. */
export function remedy(missing, {repo, branch}) {
  const checks = REQUIRED_CHECKS.map(context => ({context}));
  const body = JSON.stringify({
    name: 'main-required-checks', target: 'branch', enforcement: 'active',
    conditions: {ref_name: {include: ['~DEFAULT_BRANCH'], exclude: []}},
    rules: [{type: 'required_status_checks', parameters: {strict_required_status_checks_policy: false, required_status_checks: checks}}],
    bypass_actors: [{actor_id: 5, actor_type: 'RepositoryRole', bypass_mode: 'always'}],
  });
  return [
    `The ${branch} branch of ${repo} does not require these status checks: ${missing.map(name => `"${name}"`).join(', ')}.`,
    'This needs a repository admin (issue #1191, requirement 7). Before you apply it:',
    '  1. release.yml ignores docs-only paths, and a check that never reports stays pending. Make sure every pull request reports these checks, or keep docs-only pull requests mergeable some other way.',
    '  2. external-benchmarks.yml pushes its ledger straight to main. Keep a bypass actor for that push (repository role 5 = admin in the example), or move it to a pull request.',
    `Then run: gh api -X POST repos/${repo}/rulesets --input - <<'JSON'`,
    body,
    'JSON',
  ].join('\n');
}

async function fetchRules(repo, branch) {
  const {createEvidenceClient} = await import('./collect-test-replays.mjs');
  const client = createEvidenceClient({maxRequests: 5, aggregateBytes: 4194304, bounds: {retries: 2, timeoutMs: 30000}});
  return client.json(`repos/${repo}/rules/branches/${encodeURIComponent(branch)}?per_page=100`);
}

export async function main(args = process.argv.slice(2), {getRules = fetchRules, log = console.log, error = console.error} = {}) {
  let repo = process.env.GITHUB_REPOSITORY || 'link-assistant/formal-ai', branch = 'main';
  for (let index = 0; index < args.length; index += 2) {
    if (args[index] === '--repo') repo = args[index + 1];
    else if (args[index] === '--branch') branch = args[index + 1];
    else throw new Error(`Unknown option ${args[index]}`);
  }
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u.test(repo) || !/^[A-Za-z0-9_./-]+$/u.test(branch ?? '')) throw new Error('Invalid --repo or --branch');
  const missing = missingChecks(await getRules(repo, branch));
  if (missing.length === 0) {
    log(`required status checks on ${branch}: ${REQUIRED_CHECKS.join(', ')}`);
    return 0;
  }
  error(`::error title=Required status checks missing on ${branch}::${remedy(missing, {repo, branch}).split('\n')[0]}`);
  error(remedy(missing, {repo, branch}));
  return 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().then(code => { process.exitCode = code; }, failure => { console.error(failure.message); process.exitCode = 1; });
}
