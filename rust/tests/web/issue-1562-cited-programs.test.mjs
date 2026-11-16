// Issue #1562: a work item that only cites a Hello World repository (the
// `test-hello-world-…` addresses of a bug report) or a source file name asks
// for no program. No `main.*` is written, nothing is run or committed, and the
// summary names the reported defect. A work item that asks for the program in
// its own words ("Implement Hello World in Scala") still gets it.
// Twin of rust/tests/unit/agentic-coding/issue_1562_cited_programs.rs.
//
// The fixtures are the texts of hive-mind issues 4031, 4034 and 4035 (title,
// blank line, body). The sessions replay the recorded Agent CLI request of
// PR #1493 through the server's agentic entry
// (rust/tests/web/helpers/issue-1558-session.mjs).

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../../js/server/node-host-install.mjs';
import { runSession } from './helpers/issue-1558-session.mjs';
import { citesAnything, isCitation, obligationText } from '../../../js/agentic/work_item_citations.mjs';

const FIXTURES = 'data/benchmarks/fixtures/issue-1562';
const ISSUES = [4031, 4034, 4035];
const fixture = issue => readFileSync(`${FIXTURES}/hive-mind-${issue}.txt`, 'utf8');
const title = text => text.split('\n', 1)[0];
const bytes = text => Buffer.byteLength(text, 'utf8');
const HELLO_WORLD_ISSUES = JSON.parse(readFileSync('rust/tests/fixtures/issue-1364-hello-world-issues.json', 'utf8'));
const helloWorldIssue = language => {
  const issue = HELLO_WORLD_ISSUES.find(entry => entry.title.includes(language));
  return `${issue.title}\n\n${issue.body}`;
};

const worker = new WorkerHost();
let entry;
let unbindCitedRecipe;
let programContractAnswer;
let planWorkItemEvidence;
let workItemReport;
let workItemRecordOf;

before(async () => {
  await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
  const { agenticOutcome, chatCompletionFromPlan } = await import('../../../js/server/agentic.mjs');
  const { parseChatRequest } = await import('../../../js/server/chat-request.mjs');
  entry = { worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest };
  ({ unbindCitedRecipe } = await import('../../../js/agentic/general_execution.mjs'));
  ({ programContractAnswer } = await import('../../../js/agentic/crate/coding_program_contract.mjs'));
  ({ planWorkItemEvidence } = await import('../../../js/agentic/work_item_evidence.mjs'));
  ({ workItemReport, workItemRecordOf } = await import('../../../js/agentic/work_item_report.mjs'));
});

/** A solver like the native one: a Hello World recipe whenever its normalized text says "hello world". */
function normalizingSolver(text) {
  const normalized = text.toLowerCase().replace(/[^a-z0-9]+/gu, ' ');
  if (!normalized.includes('hello world')) return { execution_recipe: null };
  return { execution_recipe: { path: 'main.rs', language: 'rust', commands: ['rustc main.rs -o main', './main'] } };
}

describe('issue #1562 obligation text', () => {
  test('citations are blanked byte for byte and the work item keeps its own words', () => {
    for (const issue of ISSUES) {
      const text = fixture(issue);
      const obligations = obligationText(text);
      assert.equal(bytes(obligations), bytes(text), `hive-mind#${issue}: byte offsets are kept`);
      assert.equal(obligations.split('\n').length, text.split('\n').length, `hive-mind#${issue}: lines are kept`);
      assert.ok(citesAnything(text), `hive-mind#${issue} cites evidence`);
      assert.doesNotMatch(obligations, /hello/iu, `hive-mind#${issue}: Hello World appears only in citations`);
      assert.doesNotMatch(obligations, /:\/\//u, `hive-mind#${issue}: no address is left`);
      assert.ok(obligations.startsWith(title(text).replace(/`[^`]*`/gu, span => ' '.repeat(bytes(span)))), `hive-mind#${issue}: the title is an obligation`);
    }
  });

  test('each kind of citation is blanked and a quoted phrase in a sentence is kept', () => {
    const text = [
      'Implement a "Hello World" program in Scala.',
      'See https://github.com/konard/test-hello-world-01/pull/2 and <https://example.com/x>.',
      'The file `main.rs` and [the log](https://gist.example/raw/log.txt) and www.example.com.',
      '> Implement Hello World in Rust',
      '| main.rs | hello |',
      '```rust',
      'fn main() { println!("Hello, world!"); }',
      '```',
      'Done — ünïcödé stays.'
    ].join('\n');
    const obligations = obligationText(text);
    assert.equal(bytes(obligations), bytes(text));
    const lines = obligations.split('\n');
    assert.equal(lines[0], 'Implement a "Hello World" program in Scala.');
    assert.match(lines[1], /^See\s+and\s+\.$/u);
    assert.match(lines[2], /^The file\s+and\s+and\s+$/u);
    for (const index of [3, 4, 5, 6, 7]) assert.match(lines[index], /^ *$/u, `line ${index + 1} is a citation`);
    assert.equal(lines[8], 'Done — ünïcödé stays.');
    assert.equal(obligationText('Implement Hello World in Scala'), 'Implement Hello World in Scala');
    assert.equal(citesAnything('Implement Hello World in Scala'), false);
  });

  test('a requirement span wholly inside a citation is a citation', () => {
    const text = 'Fix the guard.\n\n| a | b |\n\n```js\nx();\n```\n';
    const obligations = obligationText(text);
    const span = (needle, length = bytes(needle)) => [Buffer.from(text).indexOf(needle), Buffer.from(text).indexOf(needle) + length];
    assert.equal(isCitation(obligations, span('Fix the guard.')), false);
    assert.equal(isCitation(obligations, span('| a | b |')), true);
    assert.equal(isCitation(obligations, span('```js\nx();\n```')), true);
    assert.equal(isCitation(obligations, [3, 3]), false, 'an empty span is no citation');
    assert.equal(isCitation(obligations, [0, bytes(text) + 1]), false, 'a span past the text is no citation');
  });
});

describe('issue #1562 recipe binding', () => {
  test('a recipe selected only from cited addresses is unbound and its path reported', () => {
    for (const issue of ISSUES) {
      const text = fixture(issue);
      const answer = normalizingSolver(text);
      assert.ok(answer.execution_recipe, `hive-mind#${issue}: the whole text selects Hello World, as the native solver did`);
      assert.equal(unbindCitedRecipe(answer, text, normalizingSolver), 'main.rs');
      assert.equal(answer.execution_recipe, null, 'the unbound recipe is dropped');
    }
  });

  test('a work item that asks for the program in its own words keeps its recipe', () => {
    for (const language of ['Scala', 'Kotlin', 'Rust']) {
      const text = helloWorldIssue(language);
      const answer = programContractAnswer(text);
      assert.ok(answer?.execution_recipe, `${language} Hello World selects its program`);
      const recipe = answer.execution_recipe;
      assert.equal(unbindCitedRecipe(answer, text), null, `${language} Hello World stays bound`);
      assert.equal(answer.execution_recipe, recipe);
    }
    const quoted = 'Implement a "Hello World" program in Scala.\n\nReference: https://github.com/konard/test-hello-world-01';
    const answer = normalizingSolver(quoted);
    assert.equal(unbindCitedRecipe(answer, quoted, normalizingSolver), null, 'an obligation that names the program keeps it');
    assert.ok(answer.execution_recipe);
  });

  test('the JS work-item solver selects no program for the three reports', () => {
    for (const issue of ISSUES) assert.equal(programContractAnswer(fixture(issue))?.execution_recipe ?? null, null, `hive-mind#${issue}`);
  });

  test('the evidence record and its report name the unbound write', async () => {
    const text = fixture(4031);
    // The transcript of the replayed session, after every cited file was looked up.
    const session = await runSession(entry, 4031, process.cwd(), { issueText: text, repository: 'link-assistant/hive-mind' });
    const plan = planWorkItemEvidence(text, session.messages, ['read', 'glob', 'bash'], null, 'main.rs');
    const record = workItemRecordOf(plan);
    assert.ok(record, `the evidence plan is final: ${plan?.kind}`);
    assert.deepEqual(record.unboundWrite, { path: 'main.rs', reason: 'NamedByNoObligation' });
    assert.ok(record.remainingWork.includes('unbound-write:main.rs'));
    const citations = record.requirements.filter(row => row.status === 'citation');
    assert.ok(citations.length >= 1, 'the table and the fences are citations');
    for (const row of citations) assert.match(row.text, /^(\||```)/u);
    const report = workItemReport(record, text);
    assert.match(report, /`main\.rs` was not written: no obligation of the work item names it/u);
    for (const row of citations) assert.ok(!report.includes(row.text.slice(0, 40)), `a citation is not listed as a requirement: ${row.text.slice(0, 40)}`);
    assert.doesNotMatch(report, /Created and verified/u);
    const listed = /Requirements not yet verified \((\d+)\)/u.exec(report);
    assert.equal(Number(listed?.[1]), record.requirements.length - citations.length, 'only obligations are counted');
  });
});

describe('issue #1562 sessions of the three reports', () => {
  test('a session whose only written file is named by no obligation commits nothing and claims nothing', async () => {
    // Production order (hive-mind PR 4037): `main.rs` was already written and run when the session resumes.
    const root = process.cwd();
    const exchanged = (id, name, args, content) => [
      { role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name, arguments: JSON.stringify(args) } }] },
      { role: 'tool', tool_call_id: id, content }
    ];
    const after = [
      ...exchanged('call_write_main', 'write', { filePath: `${root}/main.rs`, content: 'fn main() {\n    println!("Hello, World!");\n}\n' }, 'Wrote file successfully.'),
      ...exchanged('call_run_main', 'bash', { command: 'rustc main.rs -o main && ./main', description: 'Run the program' }, 'Hello, World!\n')
    ];
    const session = await runSession(entry, 4031, root, { issueText: fixture(4031), repository: 'link-assistant/hive-mind', after });
    assert.equal(session.stop, 'final', `the session ends with an answer: ${session.stop}`);
    for (const call of session.calls) {
      assert.ok(!['write', 'edit'].includes(call.tool), `nothing more is written: ${call.tool} ${call.arguments}`);
      assert.doesNotMatch(call.arguments, /git commit|git push|gh pr ready/u);
    }
    assert.doesNotMatch(session.final, /Created and verified/u);
    assert.equal(session.record?.authored ?? false, false);
  });

  for (const issue of ISSUES) {
    test(`hive-mind#${issue} writes, runs and commits nothing and names the defect`, async () => {
      const text = fixture(issue);
      const session = await runSession(entry, issue, process.cwd(), { issueText: text, repository: 'link-assistant/hive-mind' });
      assert.equal(session.stop, 'final', `the session ends with an answer: ${session.stop}`);
      for (const call of session.calls) {
        assert.ok(['read', 'glob'].includes(call.tool), `only reads of the cited files: ${call.tool} ${call.arguments}`);
        assert.doesNotMatch(call.arguments, /main\.\w+|git commit|gh pr ready/u);
      }
      assert.ok(session.final.includes(title(text).slice(0, 60)), 'the summary names the reported defect');
      assert.doesNotMatch(session.final, /Created and verified/u);
      assert.equal(session.record.authored, false);
      assert.ok(session.record.requirements.some(row => row.status === 'citation'), 'the cited evidence is not an obligation');
    });
  }
});
