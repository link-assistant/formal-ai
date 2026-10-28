// Retire abandoned generated fixtures without claiming that a test or feature passed.
import {generatedIssue, generatedLabels, issueSnapshot, sameIssueSnapshot} from './generated-task-custody.mjs';

export const retirementSchema = 'legacy-generated-test-retirement/v1';
export const minimumInactiveMs = 7 * 24 * 60 * 60 * 1000;
const repository = 'link-assistant/formal-ai';
let applying = false;
export function retirementDecision(issue, now = Date.now()) {
  const retain = reason => ({eligible: false, reason});
  if (!Number.isSafeInteger(now) || !generatedIssue(issue)) return retain('not-an-unmodified-generated-fixture');
  if (issue.html_url !== `https://github.com/${repository}/issues/${issue.number}` || issue.pull_request) return retain('foreign-or-pull-request');
  const labels = issueSnapshot(issue).labels;
  if (labels.length !== generatedLabels.length || !generatedLabels.every(label => labels.includes(label))) return retain('additional-labels');
  if (!Array.isArray(issue.assignees) || issue.assignees.length || issue.milestone != null || issue.locked) return retain('assigned-or-protected');
  const created = Date.parse(issue.created_at), updated = Date.parse(issue.updated_at);
  if (![created, updated].every(Number.isFinite) || updated < created || created > now || updated > now) return retain('invalid-timestamps');
  if (now - created < minimumInactiveMs || now - updated < minimumInactiveMs) return retain('recent-or-active');
  return {eligible: true, stateReason: 'not_planned', outcome: 'unverified',
    featureDeliveryCredit: false, reason: 'abandoned-generated-fixture', snapshot: issueSnapshot(issue)};
}
export function retirementPlan(issues, now = Date.now()) {
  if (!Array.isArray(issues) || issues.length >= 100 || new Set(issues.map(i => i.number)).size !== issues.length) throw Error('Incomplete or ambiguous bounded issue inventory');
  const eligible = [], retained = [];
  for (const issue of issues) {
    const decision = retirementDecision(issue, now);
    if (decision.eligible) eligible.push({number: issue.number, expectedIssue: decision.snapshot, decision});
    else retained.push({number: issue.number, reason: decision.reason});
  }
  return {schema: retirementSchema, repository, at: new Date(now).toISOString(),
    minimumInactiveDays: 7, featureDeliveryCredit: false, eligible, retained};
}
export function retirementComment(row) {
  return `<!-- formal-ai-legacy-e2e-retirement-v1 issue=${row.number} body=${row.expectedIssue.bodySha256} -->\n` +
    'Retired automated test fixture after at least seven days without activity.\n\n' +
    '**Test outcome: unverified.** This cleanup did not establish a passing historical run, so the issue is closed as **not planned**, not completed. It does not count as feature delivery.\n\n' +
    'Generated tests must record their actual outcome and close their fixtures when finished. Offline regressions retain captured test data; real GitHub tests require explicit opt-in.';
}
export async function retireGeneratedIssues({plan, client, apply = false, now = Date.now(), pause = async () => {}}) {
  if (plan?.schema !== retirementSchema || plan.repository !== repository || !Array.isArray(plan.eligible)
    || plan.eligible.length >= 100 || new Set(plan.eligible.map(r => r.number)).size !== plan.eligible.length
    || plan.eligible.some(r => !Number.isSafeInteger(r.number) || r.number < 1 || !r.expectedIssue)) throw Error('Invalid reviewed retirement plan');
  if (apply && applying) throw Error('Concurrent retirement refused');
  if (apply) applying = true;
  const results = [];
  try {
    for (const row of plan.eligible) {
      const endpoint = `repos/${repository}/issues/${row.number}`;
      let issue;
      try { issue = await client.json(endpoint); }
      catch (error) {
        results.push({number: row.number, status: 'observation-unavailable', reason: error.message});
        break;
      }
      if (!sameIssueSnapshot(row.expectedIssue, issue)) { results.push({number: row.number, status: 'retained', reason: 'changed-since-review'}); continue; }
      const decision = retirementDecision(issue, now);
      if (!decision.eligible) { results.push({number: row.number, status: 'retained', reason: decision.reason}); continue; }
      if (!apply) { results.push({number: row.number, status: 'would-retire', outcome: 'unverified'}); continue; }
      let phase = 'close';
      try {
        const patched = await client.request(endpoint, {mutation: ['--method', 'PATCH', '-f', 'state=closed', '-f', 'state_reason=not_planned']});
        if (!patched.ok) throw Error('Close response uncertain');
        const closed = await client.json(endpoint), snapshot = issueSnapshot(closed);
        if (closed.state !== 'closed' || closed.state_reason !== 'not_planned'
          || ['number', 'author', 'title', 'createdAt', 'bodySha256', 'comments', 'labels']
            .some(key => JSON.stringify(snapshot[key]) !== JSON.stringify(row.expectedIssue[key]))) throw Error('Closed issue changed');
        phase = 'comment';
        const body = retirementComment(row);
        const response = await client.request(endpoint + '/comments', {mutation: ['--method', 'POST', '-f', 'body=' + body]});
        if (!response.ok) throw Error('Outcome comment uncertain');
        const comment = JSON.parse(response.text);
        if (!Number.isSafeInteger(comment.id) || comment.body !== body) throw Error('Outcome comment not confirmed');
        results.push({number: row.number, status: 'retired', stateReason: 'not_planned',
          outcome: 'unverified', featureDeliveryCredit: false, comment: comment.id});
      } catch (error) {
        results.push({number: row.number, status: 'mutation-uncertain', phase, reason: error.message});
        break; // Never retry a write or continue a partially uncertain mutation batch.
      }
      await pause();
    }
    return {schema: retirementSchema, repository, apply, featureDeliveryCredit: false, results,
      complete: results.length === plan.eligible.length && results.every(r => !['mutation-uncertain', 'observation-unavailable'].includes(r.status))};
  } finally { if (apply) applying = false; }
}
