// Issue #921 R921-8: the languages the Hive Mind production matrix dispatched
// (Scala on the Agent leg, Kotlin on the Claude leg) are answerable, and a
// toolchain nobody ran is never reported as verified. JavaScript first: the
// browser worker answers both from their documentation captures and says the
// program was not run; rust/tests/integration/issue_412_oracle_languages.rs
// pins the native answer's "not run" status, and
// rust/tests/unit/issue_1138_toolchain_probe.rs that no catalog row may assert
// its own availability.

import assert from 'node:assert/strict';
import { before, test } from 'node:test';

import { WorkerHost } from '../../../js/server/worker-host.mjs';

let host;
before(async () => {
  host = new WorkerHost();
  await host.boot();
});

const MATRIX = [
  {
    name: 'Scala',
    language: 'scala',
    program: 'object hello {\n  def main(args: Array[String]) = {\n    println("Hello, world!")\n  }\n}',
    check: 'scalac hello.scala',
    run: 'scala hello',
  },
  {
    name: 'Kotlin',
    language: 'kotlin',
    program: 'fun main() {\n    println("Hello, world!")\n}',
    check: 'kotlinc Main.kt -include-runtime -d Main.jar',
    run: 'java -jar Main.jar',
  },
];

const notRun = (language) => `Execution status: not run - the browser sandbox cannot invoke a ${language} toolchain.\n\nCopy the snippet into a ${language} environment to verify.\n\nExpected output after verification:\n\`\`\`text\nHello, world!\n\`\`\``;

test('R921-8: the matrix languages are answered with the program and an honest not-run status', async () => {
  for (const { name, language, program, check, run } of MATRIX) {
    const result = await host.solve(`Write a hello world program in ${name}`, []);
    assert.equal(result.intent, 'write_program', name);
    assert.deepEqual(
      [result.programExecution.language, result.programExecution.checkCommand, result.programExecution.runCommand],
      [language, check, run],
    );
    assert.equal(result.programExecution.block, notRun(language));
    assert.ok(result.content.includes(`\`\`\`${language}\n${program}\n\`\`\``), result.content);
    assert.ok(result.content.includes(notRun(language)), result.content);
    assert.equal(result.content.includes('compiled and ran'), false, result.content);
    const verification = result.solverEvents.filter((event) => event.kind === 'program_verification');
    assert.equal(verification.length, 1, name);
    assert.match(verification[0].payload, new RegExp(`^language=${language} task=hello_world content_id=0x[0-9a-f]{16} verification=decomposition source=https://`));
  }
});
