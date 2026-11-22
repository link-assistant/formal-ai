// Command provenance uses publisher bytes acquired by the request (issue #1165 R1165-6).

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, test } from 'node:test';

import { programLanguages } from '../../../js/agentic/crate/coding_catalog.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { childValue, parseLino } from '../../../js/server/lino.mjs';
import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

import { codingDocumentationReplay } from './support/coding-documentation-replay.mjs';

let host;
let context;
before(async () => {
  host = new WorkerHost({}, { networkFetch: codingDocumentationReplay().networkFetch });
  context = await host.boot();
  await installNodeHost(new WorkerHost());
});

const worker = (expression) => JSON.parse(evaluate(context, `JSON.stringify(${expression})`));
const read = (path) => readFileSync(new URL(`../../../${path}`, import.meta.url), 'utf8');

const RUST_BOOK = 'https://doc.rust-lang.org/book/ch01-02-hello-world.html';
const TS_HANDBOOK = 'https://raw.githubusercontent.com/microsoft/TypeScript-Website/v2/packages/documentation/copy/en/handbook-v2/Basics.md';
const ORACLE = 'https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html';
const KOTLINLANG = 'https://kotlinlang.org/docs/command-line.html';

/** Every command the procedure rows derive, as `[language, role, command, page]`. */
const DERIVED = [
  ['rust', 'run', './main', RUST_BOOK],
  ['typescript', 'check', 'tsc hello.ts', TS_HANDBOOK],
  ['java', 'check', 'javac Main.java', ORACLE],
  ['java', 'run', 'java Main', ORACLE],
  ['kotlin', 'check', 'kotlinc Main.kt -include-runtime -d Main.jar', KOTLINLANG],
  ['kotlin', 'run', 'java -jar Main.jar', KOTLINLANG],
  ['scala', 'run', 'scala run Main.scala', 'https://docs.scala-lang.org/scala3/book/taste-hello-world.html'],
];

const acquired = (language, expression) => evaluate(context,
  `acquireDocumentationCaptures(${JSON.stringify(language)}, "hello_world").then((captures) => JSON.stringify(${expression}))`)
  .then(JSON.parse);

test('R1165-6: each command procedure binds a command from acquired bytes', async () => {
  for (const language of ['rust', 'typescript', 'java', 'kotlin', 'scala', 'go']) {
    const slug = JSON.stringify(language);
    const commands = await acquired(language, `documentedLanguageCommands(${slug}, WRITE_PROGRAM_LANGUAGES[${slug}].saveAs, captures)`);
    assert.deepEqual(commands,
      DERIVED.filter((row) => row[0] === language).map(([, role, command, source]) => ({ role, command, source })),
      language);
  }
});

test('R1165-6: the offline worker and agentic catalog agree on local commands', () => {
  const rows = worker('WRITE_PROGRAM_LANGUAGES');
  const catalog = new Map(programLanguages().map((row) => [row.slug, row]));
  for (const [language, role, command] of DERIVED) {
    assert.equal(role === 'check' ? rows[language].checkCommand : rows[language].runCommand, command, language);
    const execution = catalog.get(language).execution;
    assert.equal(role === 'check' ? execution.check_command : execution.run_command, command, language);
  }
});

test('R1165-6: a local command reaches an answer for a task without a publisher page', async () => {
  const result = await host.solve('Write a Kotlin program that prints FizzBuzz', []);
  assert.equal(result.intent, 'write_program');
  assert.match(result.content, /`kotlinc Main\.kt -include-runtime -d Main\.jar`/);
  assert.match(result.content, /`java -jar Main\.jar`/);
});

test('R1165-6: the page file name binds to the catalog stem, and a page with no such line leaves the row alone', () => {
  const policy = 'program_cache_policy\n  command_procedure\n    command\n      language kotlin\n      role check\n'
    + '      page "https://example.test/cli"\n      command_verb kotlinc\n    command\n      language kotlin\n      role run\n'
    + '      page "https://example.test/cli"\n      command_verb java\n';
  const captures = [{ url: 'https://example.test/cli', blocks: [
    { language: 'bash', text: '$ kotlinc greet.kt -d greet.jar' },
    { language: 'bash', text: '$ java -cp greet.jar GreetKt' },
  ] }];
  const raw = { 'seed/program-cache-policy.lino': policy };
  const call = (pages) => worker(`documentedLanguageCommands("kotlin", "Main.kt", ${JSON.stringify(pages)}, ${JSON.stringify(raw)})`);
  assert.deepEqual(call(captures).map((command) => command.command), ['kotlinc Main.kt -d Main.jar', 'java -cp Main.jar GreetKt']);
  assert.deepEqual(call([]), []);
});

test('R1165-6: a publisher match requires an acquired command and a registered role', async () => {
  const procedures = new Set(DERIVED.map(([language, role]) => `${language}:${role}`));
  const pairs = worker('documentationCaptureNodes().map((node) => [childValue(node, "task"), childValue(node, "language")])');
  let stated = 0;
  for (const [, language] of pairs) {
    const entries = await acquired(language, `documentedRunCommands(${JSON.stringify(language)}, captures)`);
    for (const [index, entry] of entries.entries()) {
      if (entry.documented === null) continue;
      stated += 1;
      const role = index === entries.length - 1 ? 'run' : 'check';
      assert.ok(procedures.has(`${language}:${role}`), `${language} ${role}`);
    }
  }
  assert.ok(stated >= DERIVED.length);
  const find = (node, name) => (node.name === name ? node : (node.children || []).map((child) => find(child, name)).find(Boolean));
  const record = find(parseLino(read('data/seed/program-cache-policy.lino')), 'command_procedure');
  assert.deepEqual(
    record.children.filter((node) => node.name === 'command').map((node) => `${childValue(node, 'language')}:${childValue(node, 'role') || childValue(record, 'role')}`),
    DERIVED.map(([language, role]) => `${language}:${role}`),
  );
});

test('Scala documented run binds held-out source filenames without legacy positional main', async () => {
  const command = await acquired('scala', 'documentedLanguageCommands("scala", "HeldOutUnit.scala", captures)');
  assert.deepEqual(command, [{ role: 'run', command: 'scala run HeldOutUnit.scala', source: 'https://docs.scala-lang.org/scala3/book/taste-hello-world.html' }]);
  const row = programLanguages().find((language) => language.slug === 'scala');
  assert.equal(row.execution.run_command, 'scala run Main.scala');
});

test('command role defaults are inherited while explicit roles retain authority', () => {
  const policy = 'program_cache_policy\n  command_procedure\n    role run\n    command\n      language heldout\n      page "https://publisher.test"\n      command_verb execute\n    command\n      language heldout\n      role check\n      page "https://publisher.test"\n      command_verb verify\n    command\n      language heldout\n      role invalid\n      page "https://publisher.test"\n      command_verb invalid\n';
  const rows = worker('commandProcedures(' + JSON.stringify({ 'seed/program-cache-policy.lino': policy }) + ')');
  assert.deepEqual(rows.map((row) => row.role), ['run', 'check', 'invalid']);
  assert.equal(rows.length, 3);
});
