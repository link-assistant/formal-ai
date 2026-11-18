import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import YAML from 'yaml';
import {digest} from '../../../scripts/lib/generated-task-custody.mjs';
import {retirementDecision, retirementPlan, retirementComment, retireGeneratedIssues, minimumInactiveMs}
  from '../../../scripts/lib/generated-test-retirement.mjs';

const directory = new URL('../fixtures/generated-e2e/', import.meta.url);
const capture = JSON.parse(fs.readFileSync(new URL('legacy-issues.json', directory), 'utf8'));
const now = Date.parse(capture.capturedOn + 'T23:59:59Z');
function capturedIssue(row = capture.issues[0]) {
  const body = fs.readFileSync(new URL(capture.bodyFiles[row.language], directory), 'utf8');
  assert.equal(digest(body), row.bodySha256, 'actual captured body bytes');
  return {number: row.number, state: 'open', user: {login: 'github-actions[bot]'},
    title: 'Implement Hello World in ' + row.language, labels: ['e2e-task', 'no-formal-ai-draft'],
    created_at: row.createdAt, updated_at: row.updatedAt, body, comments: 0,
    assignees: [], milestone: null, locked: false,
    html_url: `https://github.com/link-assistant/formal-ai/issues/${row.number}`};
}
function fakeClient(issues, {failComment = false} = {}) {
  const values = new Map(issues.map(i => [i.number, structuredClone(i)])), writes = [];
  return {values, writes,
    async json(endpoint) {
      const number = Number(endpoint.match(/\/issues\/([0-9]+)$/u)?.[1]);
      assert.ok(values.has(number)); return structuredClone(values.get(number));
    },
    async request(endpoint, {mutation}) {
      const number = Number(endpoint.match(/\/issues\/([0-9]+)/u)?.[1]);
      const value = values.get(number); assert.ok(value); writes.push({number, endpoint, mutation});
      if (mutation[1] === 'PATCH') {
        assert.deepEqual(mutation, ['--method', 'PATCH', '-f', 'state=closed', '-f', 'state_reason=not_planned']);
        value.state = 'closed'; value.state_reason = 'not_planned';
        return {ok: true, text: JSON.stringify(value)};
      }
      assert.equal(mutation[1], 'POST');
      if (failComment) return {ok: false, error: 'uncertain transport response'};
      value.comments++; return {ok: true, text: JSON.stringify({id: 900000 + number, body: mutation[3].slice(5)})};
    }};
}

test('all actual captured legacy bodies are recognized without inventing successful outcomes', () => {
  const issues = capture.issues.map(capturedIssue), plan = retirementPlan(issues, now);
  assert.equal(plan.eligible.length, capture.issues.length);
  assert.equal(plan.retained.length, 0);
  assert.equal(plan.featureDeliveryCredit, false);
  for (const row of plan.eligible) {
    assert.equal(row.decision.stateReason, 'not_planned');
    assert.equal(row.decision.outcome, 'unverified');
    assert.match(retirementComment(row), /does not count as feature delivery/i);
    assert.match(retirementComment(row), /not completed/i);
  }
});
test('real captured data rejects human, edited, assigned and recently active variants', () => {
  const mutations = [i => { i.user.login = 'human'; }, i => { i.body += '\nPlease keep this feature.'; },
    i => { i.comments = 1; }, i => { i.labels.push('feature'); }, i => { i.assignees = [{login: 'human'}]; },
    i => { i.milestone = {number: 1}; }, i => { i.locked = true; }, i => { i.state = 'closed'; },
    i => { i.updated_at = new Date(now).toISOString(); },
    i => { i.created_at = i.updated_at = new Date(now - minimumInactiveMs + 1).toISOString(); },
    i => { i.html_url = 'https://github.com/other/repo/issues/' + i.number; },
    i => { i.pull_request = {url: 'https://api.github.com/repos/link-assistant/formal-ai/pulls/1'}; },
    i => { delete i.assignees; }, i => { i.created_at = 'unknown'; }];
  for (const mutate of mutations) { const issue = capturedIssue(); mutate(issue); assert.equal(retirementDecision(issue, now).eligible, false); }
});
test('dry run against actual issue snapshots makes zero external writes', async () => {
  const issues = capture.issues.slice(0, 3).map(capturedIssue), client = fakeClient(issues);
  const result = await retireGeneratedIssues({plan: retirementPlan(issues, now), client, now});
  assert.equal(client.writes.length, 0); assert.ok(result.results.every(r => r.status === 'would-retire'));
});
test('retirement writes the accurate state and explanatory outcome once', async () => {
  const issue = capturedIssue(), client = fakeClient([issue]);
  const result = await retireGeneratedIssues({plan: retirementPlan([issue], now), client, apply: true, now});
  assert.equal(result.complete, true); assert.equal(result.results[0].status, 'retired');
  assert.equal(result.results[0].featureDeliveryCredit, false); assert.equal(client.writes.length, 2);
  assert.match(client.writes[1].mutation[3], /Test outcome: unverified/);
  assert.equal(client.values.get(issue.number).state_reason, 'not_planned');
});
test('changed snapshots and repeated application never perform another write', async () => {
  const issue = capturedIssue(), plan = retirementPlan([issue], now), client = fakeClient([issue]);
  client.values.get(issue.number).comments = 1;
  assert.equal((await retireGeneratedIssues({plan, client, apply: true, now})).results[0].status, 'retained');
  assert.equal(client.writes.length, 0);
  const fresh = fakeClient([issue]); await retireGeneratedIssues({plan, client: fresh, apply: true, now});
  const second = await retireGeneratedIssues({plan, client: fresh, apply: true, now});
  assert.equal(second.results[0].status, 'retained'); assert.equal(fresh.writes.length, 2);
});
test('an uncertain outcome comment stops the batch without retry or later mutations', async () => {
  const issues = capture.issues.slice(0, 2).map(capturedIssue), client = fakeClient(issues, {failComment: true});
  const result = await retireGeneratedIssues({plan: retirementPlan(issues, now), client, apply: true, now});
  assert.equal(result.complete, false); assert.equal(result.results.length, 1);
  assert.equal(result.results[0].status, 'mutation-uncertain'); assert.equal(client.writes.length, 2);
  assert.ok(client.writes.every(r => r.number === issues[0].number));
});
test('ambiguous or incomplete inventories refuse a retirement plan', () => {
  const issue = capturedIssue(); assert.throws(() => retirementPlan([issue, issue], now));
  assert.throws(() => retirementPlan(Array.from({length: 100}, (_, n) => ({...issue, number: n + 1})), now));
});
test('real GitHub effects default off and require an explicit opt-in before admission', () => {
  const w = YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/hive-original-delivery.yml', import.meta.url), 'utf8'));
  const input = w.on.workflow_dispatch.inputs['real-github-e2e'];
  assert.equal(input.type, 'boolean'); assert.equal(input.default, false);
  const gate = w.jobs.admission.if;
  assert.match(gate, /^\(vars\.FORMAL_AI_REAL_GITHUB_E2E == 'true' \|\|/u);
  assert.match(gate, /github\.event_name == 'workflow_dispatch' && inputs\['real-github-e2e'\] == true/u);
  assert.ok(gate.includes(") &&\n(github.event_name == 'workflow_dispatch' ||"));
  assert.ok(gate.includes("github.event.workflow_run.conclusion == 'success'"));
  assert.match(w.jobs.build.if, /needs\.admission\.outputs\.admitted == 'true'/u);
  for (const id of ['acceptance', 'cleanup-request']) assert.match(w.jobs[id].if, /needs\.admission\.outputs\.admitted == 'true'/u);
});
