import test from 'node:test';
import assert from 'node:assert/strict';
import {cleanupDecision, custodyFromLog, issueSnapshot, originalWorkflow, generatorFromSourceInputs, generatedTaskGenerator}
  from '../../../scripts/lib/generated-task-custody.mjs';
import {cleanupGeneratedIssues} from '../../../scripts/close-finished-test-issues.mjs';

// Full public issue body is closed-template data from pinned Hive Git blob 7f3158f0aec8d09a8199d54676b020c7dd05f59f.
const canonicalKotlinTaskBody = [
  "## Task",
  "Please implement a \"Hello World\" program in Kotlin.",
  "",
  "## Requirements",
  "1. Create a file with the appropriate extension for Kotlin",
  "2. The program should print exactly: `Hello, World!`",
  "3. Add clear comments explaining the code",
  "4. Ensure the code follows Kotlin best practices and idioms",
  "5. If applicable, include build/run instructions in a comment at the top of the file",
  "6. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**",
  "",
  "## Expected Output",
  "When the program runs, it should output:",
  "```",
  "Hello, World!",
  "```",
  "",
  "## GitHub Actions Requirements",
  "The CI/CD workflow should:",
  "- Trigger on push to main branch and on pull requests",
  "- Set up the appropriate Kotlin runtime/compiler",
  "- Run the Hello World program",
  "- Verify the output is exactly \"Hello, World!\"",
  "- Show a green check mark when tests pass",
  "",
  "Example workflow structure:",
  "- Checkout code",
  "- Setup Kotlin environment",
  "- Run the program",
  "- Assert output matches expected string",
  "",
  "## Additional Notes",
  "- The implementation should be simple and straightforward",
  "- Focus on clarity and correctness",
  "- Use the standard library only (no external dependencies unless absolutely necessary for Kotlin)",
  "- The GitHub Actions workflow should be in `ci/` directory",
  "- The workflow should have a meaningful name like `test-hello-world.yml`",
  "- Save the workflow as `ci/test-hello-world.yml`, not under `.github/workflows/`: this repository's token cannot push workflow files, so the test harness copies it there and runs it",
  "",
  "## Definition of Done",
  "- [ ] Program file created with correct extension",
  "- [ ] Code prints \"Hello, World!\" exactly",
  "- [ ] Code is properly commented",
  "- [ ] Code follows Kotlin conventions",
  "- [ ] Instructions for running the program are included (if needed)",
  "- [ ] GitHub Actions workflow created and passing",
  "- [ ] CI badge showing build status (optional but recommended)",
].join('\n');
// Synthetic adversarial controls use the real published workflow contract, not live writes.
const repository = 'link-assistant/formal-ai', head = '49f6dc896129a6c247ac13b122da0d41d8808933';
function fixture() {
  const issue = {number: 1391, state: 'open', user: {login: 'github-actions[bot]'},
    title: 'Implement Hello World in Kotlin', labels: ['e2e-task', 'no-formal-ai-draft'],
    body: canonicalKotlinTaskBody, comments: 0,
    created_at: '2026-10-21T04:41:26Z', updated_at: '2026-10-21T04:41:26Z',
    html_url: `https://github.com/${repository}/issues/1391`};
  const run = {id: 42, repository: {full_name: repository}, path: originalWorkflow,
    head_sha: head, run_attempt: 1, status: 'completed', conclusion: 'success'};
  const job = {id: 7, name: 'Actual Hive claude original delivery', status: 'completed',
    conclusion: 'success', started_at: '2026-10-21T04:40:00Z', completed_at: '2026-10-21T04:45:00Z',
    steps: [{name: 'Run actual Hive solve and original assertions', status: 'completed', conclusion: 'success'}]};
  const inputs = JSON.stringify({schema: 'formal-ai-hive-original-delivery-inputs-v1',
    hiveRepository: generatedTaskGenerator.repository, hiveCommit: generatedTaskGenerator.commit});
  const generator = generatorFromSourceInputs(inputs, head);
  const log = {ok: true, streamComplete: true, tailComplete: true,
    text: `2026-10-21T04:42:29Z    Final issue URL: ${issue.html_url}\n`};
  return {repository, issue, run, jobs: [job], inputs, generator, log,
    custody: custodyFromLog({repository, run, job, log, generator})};
}
test('complete exact generated custody permits cleanup without granting delivery authorship', () => {
  const decision = cleanupDecision(fixture());
  assert.equal(decision.eligible, true); assert.equal(decision.reason, 'completed');
  const failed = fixture(); failed.run.conclusion = 'failure'; failed.jobs[0].conclusion = 'failure';
  assert.equal(cleanupDecision(failed).reason, 'not_planned');
});
test('human feature issues, removed labels, changed bodies and added comments remain untouched', () => {
  for (const mutate of [value => { value.issue.user.login = 'human'; },
    value => { value.issue.labels.pop(); }, value => { value.issue.body = 'Please implement my feature'; },
    value => { value.issue.comments = 1; }, value => { value.issue.state = 'closed'; }]) {
    const value = fixture(); mutate(value); assert.equal(cleanupDecision(value).eligible, false);
  }
});
test('appended human requirements with the original generated prefix are never cleanup authority', () => {
  const value = fixture(); value.issue.body += '\n## Additional user requirements\nKeep and implement this feature.';
  assert.equal(cleanupDecision(value).eligible, false);
});
test('the actual per-run generator source must match the pinned complete-body authority', () => {
  const value = fixture(); value.custody.generator = {...value.generator, commit: 'e'.repeat(40)};
  assert.equal(cleanupDecision(value).eligible, false);
  assert.equal(generatorFromSourceInputs(JSON.stringify({schema: 'formal-ai-hive-original-delivery-inputs-v1',
    hiveRepository: generatedTaskGenerator.repository, hiveCommit: 'e'.repeat(40)}), head), null);
});
test('foreign workflow, source changes, active jobs and incomplete step evidence fail closed', () => {
  for (const mutate of [value => { value.run.path = '.github/workflows/other.yml'; },
    value => { value.run.repository.full_name = 'other/repo'; },
    value => { value.run.head_sha = 'a'.repeat(40); }, value => { value.run.status = 'in_progress'; },
    value => { value.jobs[0].status = 'queued'; }, value => { value.jobs[0].steps[0].status = 'pending'; },
    value => { value.issue.created_at = '2026-10-21T01:00:00Z'; }]) {
    const value = fixture(); mutate(value); assert.equal(cleanupDecision(value).eligible, false);
  }
});
test('quoted ownership, partial logs and stale source custody cannot authorize cleanup', () => {
  const value = fixture(), job = value.jobs[0];
  for (const log of [{ok: true, text: `{"content":"Final issue URL: ${value.issue.html_url}"}`},
    {ok: true, text: `Final issue URL: ${value.issue.html_url}`, truncated: true},
    {ok: false, text: `Final issue URL: ${value.issue.html_url}`, timedOut: true}]) {
    value.custody = custodyFromLog({...value, job, log});
    assert.equal(cleanupDecision(value).eligible, false);
  }
});
test('multiple distinct final issue URLs refuse ownership; repeated identical observations do not', () => {
  const value = fixture(), job = value.jobs[0], own = `Final issue URL: ${value.issue.html_url}\n`;
  for (const other of ['https://github.com/link-assistant/formal-ai/issues/1392',
    'https://github.com/other/repository/issues/1391']) {
    value.custody = custodyFromLog({...value, job,
      log: {...value.log, text: own + `Final issue URL: ${other}\n`}});
    assert.equal(cleanupDecision(value).eligible, false);
  }
  value.custody = custodyFromLog({...value, job, log: {...value.log, text: own + own}});
  assert.equal(cleanupDecision(value).eligible, true);
});
test('cancelled historical runs are retained; explicit complete owned cancelled cleanup stays honest', () => {
  const value = fixture(); value.run.conclusion = 'cancelled'; value.jobs[0].conclusion = 'cancelled';
  assert.equal(cleanupDecision(value).eligible, false);
  assert.equal(cleanupDecision({...value, allowOwnedCancelled: true}).reason, 'not_planned');
});
function emulatedClient(value, {uncertain = false} = {}) {
  const writes = [], attempts = [];
  return {writes, attempts, transferredBytes: () => 0,
    async json(endpoint) {
      if (endpoint.includes('/issues/')) return structuredClone(value.issue);
      if (endpoint.includes('/contents/')) return {encoding: 'base64', size: Buffer.byteLength(value.inputs),
        content: Buffer.from(value.inputs).toString('base64')};
      if (endpoint.includes('/jobs?')) return {total_count: value.jobs.length, jobs: structuredClone(value.jobs)};
      return structuredClone(value.run);
    },
    async request(endpoint, options) {
      if (!options.mutation) return value.log;
      writes.push({endpoint, options});
      if (uncertain) return {ok: false, timedOut: true};
      if (endpoint.endsWith('/comments')) { value.issue.comments++; return {ok: true, text: '{}'}; }
      value.issue.state = 'closed'; value.issue.updated_at = '2026-10-21T06:00:00Z';
      return {ok: true, text: JSON.stringify(value.issue)};
    }};
}
const planFor = value => ({candidates: [{number: value.issue.number,
  expectedIssue: issueSnapshot(value.issue), expectedLogSha256: value.custody.logSha256,
  runAttempt: value.run.run_attempt,
  proof: {run: value.run.id, job: value.jobs[0].id, head}}]});
test('dry-run has zero writes and yields the exact reviewed snapshot required for apply', async () => {
  const value = fixture(), client = emulatedClient(value);
  const result = await cleanupGeneratedIssues({plan: planFor(value), client});
  assert.equal(client.writes.length, 0); assert.equal(result.candidates.length, 1);
  const closed = await cleanupGeneratedIssues({plan: result, client, apply: true});
  assert.equal(closed.results[0].status, 'closed'); assert.equal(client.writes.length, 2);
});
test('fresh snapshot, run-attempt and source changes prevent every mutation', async () => {
  for (const change of [value => { value.issue.body += '\nNew user requirement'; },
    value => { value.run.run_attempt++; }, value => { value.run.head_sha = 'b'.repeat(40); }]) {
    const value = fixture(), plan = planFor(value); change(value);
    const client = emulatedClient(value);
    const result = await cleanupGeneratedIssues({plan, client, apply: true});
    assert.equal(result.results[0].status, 'retained'); assert.equal(client.writes.length, 0);
  }
});
test('edits made during log collection are observed by the final issue GET before any close', async () => {
  for (const change of [value => { value.issue.body += '\nA user added a requirement'; },
    value => { value.issue.comments++; }, value => { value.issue.labels.push('user-feature'); }]) {
    const value = fixture(), client = emulatedClient(value), original = client.request;
    client.request = async (endpoint, options) => {
      const result = await original(endpoint, options);
      if (!options.mutation) change(value);
      return result;
    };
    const result = await cleanupGeneratedIssues({plan: planFor(value), client, apply: true});
    assert.equal(result.results[0].status, 'retained');
    assert.equal(result.results[0].reason, 'issue-changed-during-owner-proof-collection');
    assert.equal(client.writes.length, 0);
  }
});
test('uncertain mutation is recorded once and never retried or followed by comments', async () => {
  const value = fixture(), client = emulatedClient(value, {uncertain: true});
  const result = await cleanupGeneratedIssues({plan: planFor(value), client, apply: true});
  assert.equal(result.results[0].status, 'mutation-uncertain'); assert.equal(client.writes.length, 1);
});
test('a changed custody log and missing reviewed run attempt refuse apply before mutation', async () => {
  const value = fixture(), client = emulatedClient(value), plan = planFor(value);
  plan.candidates[0].expectedLogSha256 = 'f'.repeat(64);
  const result = await cleanupGeneratedIssues({plan, client, apply: true});
  assert.equal(result.results[0].status, 'retained'); assert.equal(client.writes.length, 0);
  delete plan.candidates[0].runAttempt;
  await assert.rejects(cleanupGeneratedIssues({plan, client, apply: true}), /reviewed/u);
});
test('a later comment failure preserves the already observed successful close receipt', async () => {
  const value = fixture(), client = emulatedClient(value), request = client.request;
  client.request = (endpoint, options) => endpoint.endsWith('/comments')
    ? Promise.reject(new Error('bounded comment transport unavailable')) : request(endpoint, options);
  const result = await cleanupGeneratedIssues({plan: planFor(value), client, apply: true});
  assert.equal(result.results[0].status, 'closed');
  assert.equal(result.results[0].comment, 'uncertain-not-retried');
  assert.equal(value.issue.state, 'closed'); assert.equal(client.writes.length, 1);
});
test('concurrent library apply is refused while another cleanup owns the mutation lease', async () => {
  const value = fixture(), client = emulatedClient(value), read = client.json;
  let release, first = true;
  client.json = endpoint => {
    if (first) { first = false; return new Promise(resolve => { release = resolve; }); }
    return read(endpoint);
  };
  const active = cleanupGeneratedIssues({plan: planFor(value), client, apply: true, comment: false});
  await assert.rejects(cleanupGeneratedIssues({plan: planFor(value), client, apply: true}), /Concurrent/u);
  release(structuredClone(value.issue)); await active;
});
