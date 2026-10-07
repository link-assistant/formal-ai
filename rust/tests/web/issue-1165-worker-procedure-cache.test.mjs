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

test('R1165-1: a captured example that breaks the run contract is named on the miss', async () => {
  const result = await host.solve('Write a hello world program in Scala', []);
  assert.deepEqual(cacheEvents(result), [{
    kind: 'procedure_cache',
    payload: 'outcome=miss language=scala task=hello_world research_missing=reviewer_approval documentation_rejected=run_contract:Main',
  }]);
  assert.ok(result.content.includes('object Main'), 'the stored template still answers');
});

test('R1165-1: a customised request reuses the rediscovered procedure with its own literal', async () => {
  const result = await host.solve('Write a hello world program in Kotlin and replace "Hello, world!" with "Hi there"', []);
  assert.equal(result.intent, 'write_program');
  assert.deepEqual(cacheEvents(result), []);
  assert.ok(result.content.includes('fun main() {\n    println("Hi there")\n}'), result.content);
});
