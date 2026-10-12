// Issue #1165 R1165-6: no language row hard-codes a compile or run command
// that a captured documentation page states. JavaScript first: the policy
// seed's `command_procedure` rows (data/seed/program-cache-policy.lino) name,
// per language and role, the captured page and the command_verb its command
// line starts with; documentedLanguageCommands
// (js/worker/formal_ai_worker_documented_commands.js) derives the command from
// that page with the page's file name bound to the catalog's, the seed install
// fills WRITE_PROGRAM_LANGUAGES with it, and the agentic catalog
// (js/agentic/crate/coding_catalog.mjs) reads it from the worker realm. The
// Rust twins are `documented_language_commands` and `program_languages`,
// pinned by rust/tests/unit/issue_1165_documented_commands.rs.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, test } from 'node:test';

import { programLanguages } from '../../../js/agentic/crate/coding_catalog.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { childValue, parseLino } from '../../../js/server/lino.mjs';
import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

let host;
let context;
before(async () => {
  host = new WorkerHost();
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
];

const derived = (language) => worker(
  `documentedLanguageCommands(${JSON.stringify(language)}, WRITE_PROGRAM_LANGUAGES[${JSON.stringify(language)}].saveAs)`);

test('R1165-6: each command_procedure row derives the catalog command from its captured page', () => {
  for (const language of ['rust', 'typescript', 'java', 'kotlin']) {
    assert.deepEqual(
      derived(language),
      DERIVED.filter((row) => row[0] === language).map(([, role, command, source]) => ({ role, command, source })),
      language,
    );
  }
  assert.deepEqual(derived('go'), []);
});

test('R1165-6: the worker and agentic catalog rows run with the derived commands', () => {
  const rows = worker('WRITE_PROGRAM_LANGUAGES');
  const catalog = new Map(programLanguages().map((row) => [row.slug, row]));
  for (const [language, role, command] of DERIVED) {
    assert.equal(role === 'check' ? rows[language].checkCommand : rows[language].runCommand, command, language);
    const execution = catalog.get(language).execution;
    assert.equal(role === 'check' ? execution.check_command : execution.run_command, command, language);
  }
});

test('R1165-6: a derived command reaches an answer for a task the page never documented', async () => {
  const result = await host.solve('Write a Kotlin program that prints FizzBuzz', []);
  assert.equal(result.intent, 'write_program');
  assert.match(result.content, /`kotlinc Main\.kt -include-runtime -d Main\.jar`/);
  assert.match(result.content, /`java -jar Main\.jar`/);
});

test('R1165-6: the page file name binds to the catalog stem, and a page with no such line leaves the row alone', () => {
  const policy = 'program_cache_policy\n  command_procedure\n    command\n      language kotlin\n      role check\n'
    + '      page "https://example.test/cli"\n      command_verb kotlinc\n    command\n      language kotlin\n      role run\n'
    + '      page "https://example.test/cli"\n      command_verb java\n';
  const captures = 'coding_documentation_captures\n  capture\n    language "kotlin"\n    url "https://example.test/cli"\n'
    + '    block\n      language "bash"\n      code "$ kotlinc greet.kt -d greet.jar"\n'
    + '    block\n      language "bash"\n      code "$ java -cp greet.jar GreetKt"\n';
  const raw = { 'seed/program-cache-policy.lino': policy, 'seed/coding-documentation-captures.lino': captures };
  const call = (seeds) => worker(`documentedLanguageCommands("kotlin", "Main.kt", ${JSON.stringify(seeds)})`);
  assert.deepEqual(call(raw).map((command) => command.command), ['kotlinc Main.kt -d Main.jar', 'java -cp Main.jar GreetKt']);
  assert.deepEqual(call({ 'seed/program-cache-policy.lino': policy }), []);
});

/** The `role` of each entry documentedRunCommands lists: the last is the run command. */
const roles = (entries) => entries.map((_, index) => (index === entries.length - 1 ? 'run' : 'check'));

/** The text of one language row in a source table, from its opening line to the next row. */
const rowText = (source, opening, slug) => {
  const lines = source.split('\n');
  const start = lines.findIndex((line) => opening(line, slug));
  assert.notEqual(start, -1, `no row for ${slug}`);
  const end = lines.findIndex((line, index) => index > start && /^\s*(\w+: \{|ProgramLanguage \{|language \w+)$/.test(line));
  return lines.slice(start, end === -1 ? undefined : end).join('\n');
};

test('R1165-6: no language row hard-codes a command a captured page states', () => {
  const procedures = new Set(DERIVED.map(([language, role]) => `${language}:${role}`));
  const pairs = worker('documentationCaptureNodes().map((node) => [childValue(node, "task"), childValue(node, "language")])');
  let stated = 0;
  for (const [task, language] of pairs) {
    const entries = worker(`documentedRunCommands(${JSON.stringify(task)}, ${JSON.stringify(language)})`);
    roles(entries).forEach((role, index) => {
      if (entries[index].documented === null) return;
      stated += 1;
      assert.ok(procedures.has(`${language}:${role}`), `${language} ${role}: its page states it, so a command_procedure row must derive it`);
    });
  }
  assert.ok(stated >= DERIVED.length);
  const find = (node, name) => (node.name === name ? node : (node.children || []).map((child) => find(child, name)).find(Boolean));
  const record = find(parseLino(read('data/seed/program-cache-policy.lino')), 'command_procedure');
  assert.deepEqual(
    record.children.filter((node) => node.name === 'command').map((node) => `${childValue(node, 'language')}:${childValue(node, 'role')}`),
    DERIVED.map(([language, role]) => `${language}:${role}`),
  );
  const tables = [
    [read('rust/src/coding/catalog/languages.rs'), (line, slug) => line.trim() === `slug: "${slug}",`],
    [read('js/worker/formal_ai_worker_installation_and_software_followups.js'), (line, slug) => line === `  ${slug}: {`],
    [read('data/meta/agentic-coding-catalog.lino'), (line, slug) => line === `  language ${slug}`],
  ];
  for (const [language, , command] of DERIVED) {
    for (const [source, opening] of tables) {
      const row = rowText(source, opening, language);
      assert.ok(!row.includes(`"${command}"`), `${language} row still states "${command}"`);
    }
  }
});
