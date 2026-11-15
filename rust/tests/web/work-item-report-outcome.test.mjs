// Issue #1560 outcome: a Hive Mind session on issues 1486, 1489 and 1501
// reads repository files instead of refusing every read; a work-item final is
// a localized report, never a JSON object; a fetched issue page yields only its
// title and body; a page without a body is rejected and the API read is used;
// a user turn that forbids a read is still honoured.
// Native twin: rust/tests/unit/agentic-coding/work_item_report_outcome.rs.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { workItemPageText } from '../../../js/agentic/work_item_page.mjs';
import { workItemRecordOf, parsesAsJsonObject, workItemReport } from '../../../js/agentic/work_item_report.mjs';
import { completeOwnedSourceReadFrame } from '../../../js/agentic/tool_result.mjs';

const root = new URL('../../../', import.meta.url).pathname;
const fixtures = join(root, 'data/benchmarks/fixtures/work-item-1560');
const issues = JSON.parse(readFileSync(join(fixtures, 'issues.json'), 'utf8')).issues;
const page1501 = readFileSync(join(fixtures, 'page-1501.txt'), 'utf8');
const signIn = readFileSync(join(fixtures, 'signin.txt'), 'utf8');
// Hive Mind's tool set and user turn, recorded from the pull request 1502 session.
const tools = ['bash', 'read', 'glob', 'grep', 'list', 'edit', 'write', 'task', 'webfetch', 'websearch', 'codesearch', 'batch', 'todowrite', 'todoread'];
const chrome = ['· GitHub', 'Skip to content', 'Sign in'];

before(async () => { await installNodeHost(new WorkerHost({})); });

function prompt(number) {
  return readFileSync(join(fixtures, 'hive-mind-prompt.txt'), 'utf8')
    .split('{number}').join(String(number)).split('{pull}').join(String(number + 1)).split('{branch}').join('0123456789ab');
}

/** A repository file of this checkout, or `null`. */
function repositoryFile(path) {
  const full = join(root, String(path ?? '').replace(/^\/+/, ''));
  return typeof path === 'string' && path !== '' && existsSync(full) && statSync(full).isFile() ? readFileSync(full, 'utf8') : null;
}

/**
 * Drives a session to its final answer. `answer(tool, args)` returns a tool
 * result's content, or `undefined` for the default: a repository file in the
 * agent's numbered read frame for a read, empty output for anything else.
 * `read` lists the repository files the session read.
 */
async function drive(messages, selectedTools, answer, steps = 24) {
  const transcript = [];
  const read = [];
  for (let step = 0; step < steps; step += 1) {
    const plan = await planChatStepResolved(messages, selectedTools);
    if (plan?.kind !== 'tool_calls') return { plan, transcript, read };
    const call = plan.calls[0];
    const args = JSON.parse(call.arguments);
    transcript.push([call.tool, args]);
    const path = args.filePath ?? args.file_path ?? args.path;
    const file = call.tool === 'read' ? repositoryFile(path) : null;
    if (file !== null) read.push(path);
    const content = answer(call.tool, args)
      ?? (call.tool === 'read' ? (file === null ? 'Error: File not found: ' + path : completeOwnedSourceReadFrame(file)) : '');
    const id = 'outcome-' + step;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
      { role: 'tool', tool_call_id: id, name: call.tool, content });
  }
  assert.fail('the session did not end within ' + steps + ' steps: ' + JSON.stringify(transcript));
}

function issueRead(number, page = null) {
  const issue = issues[number];
  return (tool, args) => {
    if (tool === 'bash' && /gh (issue|api) /.test(args.command ?? '')) return issue.title + '\n\n' + issue.body + '\n\n__formal_ai_exit=0\n';
    if (tool === 'webfetch') return page ?? issue.title + '\n\n' + issue.body;
    return undefined;
  };
}

/** A work-item final is a localized report; its record stays on the final result. */
function reported(plan) {
  assert.equal(plan?.kind, 'final', JSON.stringify(plan));
  assert.equal(parsesAsJsonObject(plan.answer), false, plan.answer);
  const record = workItemRecordOf(plan);
  assert.ok(record, 'a work-item final keeps its record');
  assert.notEqual(record.reason, 'ReadPolicyRefused', plan.answer);
  assert.equal(plan.answer.includes('ReadPolicyRefused'), false, plan.answer);
  return record;
}

for (const number of ['1486', '1489', '1501']) {
  test('issue ' + number + ': the Hive Mind session reads a repository file and does not refuse the read', async () => {
    const { plan, transcript, read } = await drive([{ role: 'user', content: prompt(Number(number)) }], tools, issueRead(number));
    assert.equal(transcript[0][0], 'bash');
    assert.ok(transcript[0][1].command.includes('issues/' + number), 'the issue is read first');
    assert.ok(read.length > 0, 'no repository file was read: ' + JSON.stringify(transcript));
    const record = reported(plan);
    assert.ok(record.observations.some(observation => observation.complete && read.includes(observation.path)), JSON.stringify(record.observations));
    assert.ok(plan.answer.includes('`' + read[0] + '`'), 'the report names the file it read: ' + plan.answer);
    assert.ok(record.requirements.length > 1, 'the issue text is split into requirements');
  });
}

test('issue 1486: a link into .github/workflows/ is read at its linked path and nothing is left unresolved', async () => {
  const { plan, read } = await drive([{ role: 'user', content: prompt(1486) }], tools, issueRead('1486'));
  assert.ok(read.includes('.github/workflows/hive-mind-draft.yml'), JSON.stringify(read));
  const record = reported(plan);
  assert.deepEqual(record.unresolved, [], JSON.stringify(record.unresolved));
  assert.equal(record.reason, 'EvidenceCollected', plan.answer);
});

test('issue 1501: a bare README.md is the root file, not a deeper one whose directory shares a word with the text', async () => {
  const deeper = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/compiled-first-slice/README.md';
  const read1501 = issueRead('1501');
  const answer = (tool, args) => (tool === 'glob' && args.pattern === '**/README.md' ? deeper + '\nREADME.md\n' : read1501(tool, args));
  const { plan, read } = await drive([{ role: 'user', content: prompt(1501) }], tools, answer);
  assert.ok(read.includes('README.md'), JSON.stringify(read));
  assert.equal(read.includes(deeper), false, JSON.stringify(read));
  reported(plan);
});

test('issue 1501 page: only the title and body are kept as the requirement text', () => {
  const text = workItemPageText(page1501);
  assert.ok(text.startsWith('Agent mode 0.355.27 misroutes common file requests'), text.slice(0, 200));
  assert.ok(text.includes('Summary'));
  for (const marker of chrome) assert.equal(text.includes(marker), false, marker);
  assert.equal(text.includes('Sign up for free'), false);
  assert.equal(text.includes('&#39;'), false, 'entities are decoded');
});

test('issue 1501 page: a session given only the page reads a file and its requirements carry no chrome', async () => {
  const messages = [{ role: 'user', content: prompt(1501) }];
  const fetchOnly = tools.filter(tool => tool !== 'bash');
  const { plan, transcript, read } = await drive(messages, fetchOnly, issueRead('1501', page1501));
  assert.equal(transcript[0][0], 'webfetch', JSON.stringify(transcript));
  assert.ok(read.length > 0, 'no repository file was read: ' + JSON.stringify(transcript));
  const record = reported(plan);
  assert.ok(record.requirements[0].text.startsWith('Agent mode 0.355.27 misroutes common file requests'), record.requirements[0].text);
  for (const requirement of record.requirements) {
    for (const marker of chrome) assert.equal(requirement.text.includes(marker), false, marker + ' in ' + requirement.text);
  }
  for (const marker of chrome) assert.equal(plan.answer.includes(marker), false, marker);
});

test('a page without an issue body is rejected and the API read is used instead', async () => {
  assert.equal(workItemPageText(signIn), null);
  const messages = [{ role: 'user', content: prompt(1501) }];
  const used = [];
  const answer = (tool, args) => {
    used.push(tool === 'bash' ? args.command : tool);
    if (tool === 'webfetch') return signIn;
    if (tool === 'bash' && /gh issue view/.test(args.command ?? '')) return 'gh: To get started with GitHub CLI, please run:  gh auth login\n\n__formal_ai_exit=4\n';
    if (tool === 'bash' && /(curl|gh api) /.test(args.command ?? '')) return issues['1501'].title + '\n\n' + issues['1501'].body + '\n\n__formal_ai_exit=0\n';
    return undefined;
  };
  const { plan, transcript, read } = await drive(messages, tools, answer);
  const webfetch = used.indexOf('webfetch');
  assert.ok(webfetch >= 0, JSON.stringify(used));
  const fallback = used.findIndex((command, index) => index > webfetch && /(curl|gh api) /.test(command));
  assert.ok(fallback > webfetch, 'the REST or gh api read follows the rejected page: ' + JSON.stringify(used));
  assert.ok(read.length > 0, 'the session continues from the API body: ' + JSON.stringify(transcript));
  const record = reported(plan);
  for (const requirement of record.requirements) assert.equal(requirement.text.includes('Sign in'), false, requirement.text);
});

test('a user turn that says not to read a file is still honoured', async () => {
  const messages = [{ role: 'user', content: 'Do not read package.json. Investigate package.json.' }];
  const plan = await planChatStepResolved(messages, ['read']);
  const reads = plan?.kind === 'tool_calls' ? plan.calls.filter(call => call.tool === 'read' && call.arguments.includes('package.json')) : [];
  assert.deepEqual(reads, []);
  assert.equal(plan === null || parsesAsJsonObject(plan.answer ?? ''), plan === null ? true : false);
});

test('a refusal names the requirement it concerns in the localized report', () => {
  const record = {
    schema: 'repository-work-item-evidence/v1', reason: 'ReadPolicyRefused', authored: false, observations: [],
    requirements: [{ text: 'Keep the release notes short.' }, { text: 'Do not read secrets/token.txt.\nIt holds a credential.' }],
    refused: { path: 'secrets/token.txt', requirement: 1 },
  };
  const report = workItemReport(record, 'Solve the issue.');
  assert.equal(parsesAsJsonObject(report), false);
  assert.match(report, /secrets\/token\.txt/);
  assert.match(report, /Do not read secrets\/token\.txt\. It holds a credential\./);
  assert.match(report, /ReadPolicyRefused/);
});
