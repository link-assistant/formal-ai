// Issue #1165 R1165-1: data/seed/coding-documentation-captures.lino is
// pre-cached source data, not a store of programs. Every capture's header
// names a byte-for-byte fixture and its SHA-256; every block row is what the
// page formalizer reads from those bytes, so the seed is re-derived here from
// the fixtures and must match byte for byte (with its rust/embedded mirror).
// The native twin is `documentation_captures_are_the_formalized_fixtures` in
// rust/tests/unit/issue_1165_discovery_production.rs.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';

import {
  EMBEDDED_PATH,
  SEED_PATH,
  captureHeaders,
  renderSeed,
} from '../../../scripts/generate-coding-documentation-captures.mjs';
import { REPO_ROOT, createWorkerContext, evaluate } from '../../../js/server/worker-host.mjs';

const context = createWorkerContext();
const seeded = evaluate(context, 'loadSeed()');
const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), 'utf8');

test('the captures seed is the formalized fixtures, byte for byte', async () => {
  await seeded;
  const seed = read(SEED_PATH);
  const headers = captureHeaders(context, seed);
  assert.deepEqual(
    headers.map((header) => [header.language, header.url]),
    [
      ['rust', 'https://doc.rust-lang.org/book/ch01-02-hello-world.html'],
      ['go', 'https://go.dev/doc/tutorial/getting-started'],
      ['kotlin', 'https://kotlinlang.org/docs/command-line.html'],
      ['kotlin', 'https://kotlinlang.org/docs/kotlin-tour-hello-world.html'],
      ['scala', 'https://docs.scala-lang.org/scala3/book/taste-hello-world.html'],
      ['python', 'https://wiki.python.org/moin/BeginnersGuide/Programmers/SimpleExamples'],
      ['javascript', 'https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/api/console/index.md'],
      ['typescript', 'https://raw.githubusercontent.com/microsoft/TypeScript-Website/v2/packages/documentation/copy/en/handbook-v2/Basics.md'],
      ['c', 'https://raw.githubusercontent.com/MicrosoftDocs/cpp-docs/main/docs/c-runtime-library/reference/puts-putws.md'],
      ['cpp', 'https://learn.microsoft.com/en-us/cpp/build/vscpp-step-1-create'],
      ['csharp', 'https://learn.microsoft.com/en-us/dotnet/csharp/tour-of-csharp/tutorials/hello-world'],
      ['ruby', 'https://www.ruby-lang.org/en/examples/hello_world/'],
      ['swift', 'https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/GuidedTour/GuidedTour.md'],
      ['java', 'https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html'],
      ['php', 'https://www.php.net/manual/en/tutorial.firstpage.php'],
    ],
  );
  assert.equal(renderSeed(context, headers), seed, 'regenerate with scripts/generate-coding-documentation-captures.mjs --write');
  assert.equal(read(EMBEDDED_PATH), seed, 'the embedded mirror is byte-identical');
});

test('the worker reads the captures the seed records', async () => {
  await seeded;
  const kotlin = evaluate(context, 'documentationCaptures("kotlin", "hello_world").map((capture) => capture.url)');
  assert.deepEqual([...kotlin], ['https://kotlinlang.org/docs/command-line.html', 'https://kotlinlang.org/docs/kotlin-tour-hello-world.html']);
  assert.equal(evaluate(context, 'documentationCaptures("r", "hello_world").length'), 0);
  assert.equal(evaluate(context, 'documentationRouteActive()'), true);
});

// R1165-6: a captured line that is a catalog command with the documented file
// name bound to the catalog's states that command; the rest are listed with
// no documented line, which records the languages whose captured pages state
// none of their catalog commands.
test('R1165-6: the documented check and run commands of every captured language', async () => {
  await seeded;
  const documented = (language) => [...evaluate(context, `documentedRunCommands("hello_world", ${JSON.stringify(language)})`)]
    .map((command) => [command.catalog, command.documented]);
  assert.deepEqual(documented('rust'), [['rustc main.rs -o main', null], ['./main', './main']]);
  assert.deepEqual(documented('kotlin'), [
    ['kotlinc Main.kt -include-runtime -d Main.jar', 'kotlinc hello.kt -include-runtime -d hello.jar'],
    ['java -jar Main.jar', 'java -jar hello.jar'],
  ]);
  assert.deepEqual(documented('typescript'), [['tsc hello.ts', 'tsc hello.ts'], ['node hello.js', null]]);
  assert.deepEqual(documented('go'), [['go run main.go', null]]);
  assert.deepEqual(documented('scala'), [['scalac Main.scala', null], ['scala Main', null]]);
  assert.deepEqual(documented('python'), [['python3 -m py_compile main.py', null], ['python3 main.py', null]]);
  assert.deepEqual(documented('javascript'), [['node --check main.js', null], ['node main.js', null]]);
  assert.deepEqual(documented('c'), [['gcc main.c -o main', null], ['./main', null]]);
  assert.deepEqual(documented('cpp'), [['g++ main.cpp -o main', null], ['./main', null]]);
  assert.deepEqual(documented('csharp'), [['dotnet build', null], ['dotnet run', null]]);
  assert.deepEqual(documented('ruby'), [['ruby -c main.rb', null], ['ruby main.rb', null]]);
  assert.deepEqual(documented('java'), [['javac Main.java', 'javac HelloWorldApp.java'], ['java Main', 'java HelloWorldApp']]);
  assert.deepEqual(documented('php'), [['php -l main.php', null], ['php main.php', null]]);
});

// R1165-6: the run contract a documented program binds. The catalog's file
// stem stays when the program declares what the commands invoke (Kotlin);
// otherwise the name a captured command states (Java's HelloWorldApp) or the
// name after an entry_container keyword (Scala's object hello) binds in the
// file and every command; each command names its source.
test('R1165-6: the documented run contract of every captured language', async () => {
  await seeded;
  const contract = (language) => evaluate(context, `JSON.stringify(documentedProgram("hello_world", ${JSON.stringify(language)}).contract)`);
  const kotlinlang = 'https://kotlinlang.org/docs/command-line.html';
  const oracle = 'https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html';
  assert.deepEqual(JSON.parse(contract('java')), { saveAs: 'HelloWorldApp.java', commands: [
    { role: 'check', command: 'javac HelloWorldApp.java', source: oracle },
    { role: 'run', command: 'java HelloWorldApp', source: oracle },
  ] });
  assert.deepEqual(JSON.parse(contract('scala')), { saveAs: 'hello.scala', commands: [
    { role: 'check', command: 'scalac hello.scala', source: 'catalog' },
    { role: 'run', command: 'scala hello', source: 'catalog' },
  ] });
  assert.deepEqual(JSON.parse(contract('kotlin')), { saveAs: 'Main.kt', commands: [
    { role: 'check', command: 'kotlinc Main.kt -include-runtime -d Main.jar', source: kotlinlang },
    { role: 'run', command: 'java -jar Main.jar', source: kotlinlang },
  ] });
  assert.deepEqual(JSON.parse(contract('rust')).commands.map((entry) => entry.source),
    ['catalog', 'https://doc.rust-lang.org/book/ch01-02-hello-world.html']);
});

// The deviation a documented program carries that its verification cannot
// see: php.net's echo prints no line break and its line has no PHP_EOL.
test('R1165-1: a documented program printing no trailing newline records the deviation', async () => {
  await seeded;
  const deviations = ['rust', 'python', 'javascript', 'typescript', 'go', 'c', 'cpp', 'csharp', 'ruby', 'kotlin', 'java', 'scala', 'php', 'swift']
    .map((language) => [language, evaluate(context, `documentedProgram("hello_world", ${JSON.stringify(language)}).deviation`)])
    .filter((pair) => pair[1] !== null);
  assert.deepEqual(deviations, [['php', 'trailing_newline=absent']]);
});

test('R1165-6: a documented command binds the documented file name consistently', async () => {
  await seeded;
  const matches = (documented, catalog, saveAs) =>
    evaluate(context, `documentedCommandMatches(${JSON.stringify(documented)}, ${JSON.stringify(catalog)}, ${JSON.stringify(saveAs)})`);
  assert.equal(matches('kotlinc hello.kt -d hello.jar', 'kotlinc Main.kt -d Main.jar', 'Main.kt'), true);
  assert.equal(matches('kotlinc hello.kt -d world.jar', 'kotlinc Main.kt -d Main.jar', 'Main.kt'), false);
  assert.equal(matches('rustc main.rs', 'rustc main.rs -o main', 'main.rs'), false);
  assert.equal(matches('go run .', 'go run main.go', 'main.go'), false);
});

// R1165-4: the documentation route is a procedure discovery found, so a
// language whose captures rediscover a verified program is known.
test('R1165-4: the documentation route knows every language its captures rediscover', async () => {
  await seeded;
  const known = ['rust', 'python', 'javascript', 'typescript', 'go', 'c', 'cpp', 'csharp', 'ruby', 'kotlin', 'swift', 'scala', 'java', 'php', 'r', 'laravel']
    .filter((language) => evaluate(context, `documentationKnowsLanguage(${JSON.stringify(language)})`));
  assert.deepEqual(known, ['rust', 'python', 'javascript', 'typescript', 'go', 'c', 'cpp', 'csharp', 'ruby', 'kotlin', 'swift', 'scala', 'java', 'php']);
});
