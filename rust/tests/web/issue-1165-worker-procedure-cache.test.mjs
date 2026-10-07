// Issue #1165 R1165-10: the browser worker's catalog `write_program` arm logs
// the `procedure_cache` event the native `WriteProgram` branch of
// rust/src/solver.rs appends. The worker ships no cache rows (the committed
// data/cache/coding-procedure-cache.lino is empty). An unmodified catalog
// request whose pair has documentation captures rediscovers its program from
// them (R1165-1, `outcome=discovered`); any other unmodified request is the
// miss, with `research_missing` read from the miss_route rows of
// data/seed/program-cache-policy.lino and the documentation rejection named
// when captured pages yielded nothing; a request that customised the template
// is not answered from the cache and logs no `procedure_cache` event.

import assert from 'node:assert/strict';
import { before, test } from 'node:test';

import { contentAddress, missResearchMissing } from '../../../js/agentic/crate/discovery_production.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

let host;
before(async () => {
  host = new WorkerHost();
  await host.boot();
  await installNodeHost(new WorkerHost());
});

const cacheEvents = (result) => (result.solverEvents || []).filter((event) => event.kind === 'procedure_cache');

test('the worker and the agentic root read the same miss_route gap', () => {
  assert.deepEqual(missResearchMissing(), ['reviewer_approval']);
});

// Python counting to three has no documentation capture (its output is not
// one printed literal), so its unmodified request is the research miss.
test('an unmodified catalog request logs the exact native procedure_cache miss', async () => {
  const result = await host.solve('Write a Python program that counts to three', []);
  assert.equal(result.intent, 'write_program');
  assert.deepEqual(cacheEvents(result), [
    { kind: 'procedure_cache', payload: 'outcome=miss language=python task=count_to_three research_missing=reviewer_approval' },
  ]);
  const kinds = result.solverEvents.map((event) => event.kind);
  assert.equal(kinds.indexOf('procedure_cache'), kinds.indexOf('legacy_intent') + 1);
});

test('a request that customised the template logs no procedure_cache event', async () => {
  const result = await host.solve('Write a hello world program in Python and replace "Hello, world!" with "Hi there"', []);
  assert.equal(result.intent, 'write_program');
  assert.deepEqual(cacheEvents(result), []);
});

test('a non-program answer logs no procedure_cache event', async () => {
  assert.deepEqual(cacheEvents(await host.solve('What is 2 + 2?', [])), []);
});

const hex = (entry) => `0x${contentAddress(entry).toString(16).padStart(16, '0')}`;
const DISCOVERED = [
  ['Rust', 'rust', 'https://doc.rust-lang.org/book/ch01-02-hello-world.html', 'fn main() {\n    println!("Hello, world!");\n}'],
  ['Python', 'python', 'https://wiki.python.org/moin/BeginnersGuide/Programmers/SimpleExamples', "print('Hello, world!')"],
  ['JavaScript', 'javascript', 'https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/api/console/index.md',
    'console.log("Hello, world!");'],
  ['TypeScript', 'typescript',
    'https://raw.githubusercontent.com/microsoft/TypeScript-Website/v2/packages/documentation/copy/en/handbook-v2/Basics.md',
    '// Greets the world.\nconsole.log("Hello, world!");'],
  ['Go', 'go', 'https://go.dev/doc/tutorial/getting-started',
    'package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello, world!")\n}'],
  ['C', 'c', 'https://raw.githubusercontent.com/MicrosoftDocs/cpp-docs/main/docs/c-runtime-library/reference/puts-putws.md',
    '// crt_puts.c\n// This program uses puts to write a string to stdout.\n\n#include <stdio.h>\n\nint main( void )\n{\n   puts( "Hello, world!" );\n}'],
  ['C++', 'cpp', 'https://learn.microsoft.com/en-us/cpp/build/vscpp-step-1-create',
    '#include <iostream>\n\nint main()\n{\n    std::cout << "Hello, world!" << std::endl;\n    return 0;\n}'],
  ['C#', 'csharp', 'https://learn.microsoft.com/en-us/dotnet/csharp/tour-of-csharp/tutorials/hello-world',
    'Console.WriteLine("Hello, world!");'],
  ['Ruby', 'ruby', 'https://www.ruby-lang.org/en/examples/hello_world/',
    '# The famous Hello World\n# Program is trivial in\n# Ruby. Superfluous:\n#\n# * A "main" method\n# * Newline\n# * Semicolons\n#\n# Here is the Code:\n\nputs "Hello, world!"'],
  ['Kotlin', 'kotlin', 'https://kotlinlang.org/docs/command-line.html', 'fun main() {\n    println("Hello, world!")\n}'],
  ['Java', 'java', 'https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html',
    '/**\n * The HelloWorldApp class implements an application that\n * simply prints "Hello World!" to standard output.\n */\nclass HelloWorldApp {\n    public static void main(String[] args) {\n        System.out.println("Hello, world!"); // Display the string.\n    }\n}'],
  ['Scala', 'scala', 'https://docs.scala-lang.org/scala3/book/taste-hello-world.html',
    'object hello {\n  def main(args: Array[String]) = {\n    println("Hello, world!")\n  }\n}'],
  ['PHP', 'php', 'https://www.php.net/manual/en/tutorial.firstpage.php', '<?php\n\necho "Hello, world!";\n\n?>'],
];

test('R1165-1: a pair with documentation captures answers from the program rediscovered from them', async () => {
  const context = await host.boot();
  for (const [name, language, source, program] of DISCOVERED) {
    const result = await host.solve(`Write a hello world program in ${name}`, []);
    assert.equal(result.intent, 'write_program', name);
    assert.deepEqual(cacheEvents(result), [{
      kind: 'procedure_cache',
      payload: `outcome=discovered language=${language} task=hello_world rediscovery_source=${source} content_id=${hex(program)}`,
    }], name);
    assert.ok(result.content.includes(program), result.content);
    // R1165-4: the worker stores no program for the pair; the captures supply it.
    assert.equal(evaluate(context, `WRITE_PROGRAM_TEMPLATES.hello_world[${JSON.stringify(language)}]`), undefined, name);
  }
});

const documentationEvents = (result) => (result.solverEvents || [])
  .filter((event) => event.kind === 'command_source' || event.kind === 'documentation_deviation');

// R1165-6 and the Java/Scala binding: the documented class or object name
// binds the file the answer saves and every command, a command a captured
// page states is shown from that page, and the derivation records the
// source of each command and the deviation a documented program carries.
test('R1165-6: the documented run contract binds the program name and records each command source', async () => {
  const oracle = 'https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html';
  const java = await host.solve('Write a hello world program in Java', []);
  assert.deepEqual(documentationEvents(java), [
    { kind: 'command_source', payload: `language=java task=hello_world role=check source=${oracle} command=javac HelloWorldApp.java` },
    { kind: 'command_source', payload: `language=java task=hello_world role=run source=${oracle} command=java HelloWorldApp` },
  ]);
  assert.ok(java.content.includes('Save the code above to a file named `HelloWorldApp.java`.'), java.content);
  assert.deepEqual([java.programExecution.checkCommand, java.programExecution.runCommand], ['javac HelloWorldApp.java', 'java HelloWorldApp']);
  const scala = await host.solve('Write a hello world program in Scala', []);
  assert.deepEqual(documentationEvents(scala), [
    { kind: 'command_source', payload: 'language=scala task=hello_world role=check source=catalog command=scalac hello.scala' },
    { kind: 'command_source', payload: 'language=scala task=hello_world role=run source=catalog command=scala hello' },
  ]);
  const php = await host.solve('Write a hello world program in PHP', []);
  assert.deepEqual(documentationEvents(php), [
    { kind: 'command_source', payload: 'language=php task=hello_world role=check source=catalog command=php -l main.php' },
    { kind: 'command_source', payload: 'language=php task=hello_world role=run source=catalog command=php main.php' },
    { kind: 'documentation_deviation', payload: 'language=php task=hello_world trailing_newline=absent' },
  ]);
  const kinds = php.solverEvents.map((event) => event.kind);
  assert.equal(kinds.indexOf('command_source'), kinds.indexOf('procedure_cache') + 1);
});

// The derivation names what verified each documented program: a recorded
// harness run only when the program's content id is the one that run
// executed (the Rust Book's equals the template the issue-8 harness ran),
// otherwise its page and the decomposition check.
test('R1165-1: program_verification names the recorded run or the page', async () => {
  const verification = async (prompt) => (await host.solve(prompt, [])).solverEvents
    .filter((event) => event.kind === 'program_verification').map((event) => event.payload);
  const rust = 'fn main() {\n    println!("Hello, world!");\n}';
  assert.deepEqual(await verification('Write a hello world program in Rust'), [
    `language=rust task=hello_world content_id=${hex(rust)} verification=recorded source=issue-8 local verification harness (isolated sandbox)`,
  ]);
  assert.deepEqual(await verification('Write a hello world program in Python'), [
    `language=python task=hello_world content_id=${hex("print('Hello, world!')")} verification=decomposition source=https://wiki.python.org/moin/BeginnersGuide/Programmers/SimpleExamples`,
  ]);
});

// R1165-4: the coding oracle reads the documentation route before its cached
// snapshots, so Swift (no catalog program in either runtime) is answered from
// the Swift book and its Hello World Collection snapshot is retired.
test('R1165-4: the oracle answers Swift from the captured Swift book', async () => {
  const context = await host.boot();
  const swift = await host.solve('write me a hello world program in swift', []);
  assert.equal(swift.intent, 'write_program_oracle_hello_world_swift');
  assert.equal(swift.content, 'Here is a minimal Swift program (hello world):\n\n```swift\nprint("Hello, world!")\n// Prints "Hello, world!"\n```\n\nOutput:\n```text\nHello, world!\n```\nSource: Documentation capture (https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/GuidedTour/GuidedTour.md), cached locally as a popular example.');
  assert.equal(evaluate(context, 'CODING_ORACLE_SNAPSHOTS.some((snippet) => snippet.languageSlug === "swift")'), false);
});

test('R1165-1: a customised request reuses the rediscovered procedure with its own literal', async () => {
  const result = await host.solve('Write a hello world program in Kotlin and replace "Hello, world!" with "Hi there"', []);
  assert.equal(result.intent, 'write_program');
  assert.deepEqual(cacheEvents(result), []);
  assert.ok(result.content.includes('fun main() {\n    println("Hi there")\n}'), result.content);
});
