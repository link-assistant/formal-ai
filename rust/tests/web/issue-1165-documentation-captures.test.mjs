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
    ],
  );
  assert.equal(renderSeed(context, headers), seed, 'regenerate with scripts/generate-coding-documentation-captures.mjs --write');
  assert.equal(read(EMBEDDED_PATH), seed, 'the embedded mirror is byte-identical');
});

test('the worker reads the captures the seed records', async () => {
  await seeded;
  const kotlin = evaluate(context, 'documentationCaptures("kotlin", "hello_world").map((capture) => capture.url)');
  assert.deepEqual([...kotlin], ['https://kotlinlang.org/docs/command-line.html', 'https://kotlinlang.org/docs/kotlin-tour-hello-world.html']);
  assert.equal(evaluate(context, 'documentationCaptures("java", "hello_world").length'), 0);
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
  assert.deepEqual(known, ['rust', 'python', 'javascript', 'typescript', 'go', 'c', 'cpp', 'csharp', 'ruby', 'kotlin', 'swift']);
});
