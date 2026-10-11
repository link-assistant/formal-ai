// Issue #1185 R1, R2 and R7 in the JavaScript root: the one shape table
// (data/seed/diagnostic-code-shapes.lino) formalizes a diagnostic of each of
// the fourteen emitted languages through the generic `{slot}` matcher of
// js/agentic/repair_loop.mjs. The table is the one
// rust/tests/unit/agentic-coding/issue_1185_error_repair_loop.rs
// (`every_emitted_language_formalizes_through_the_one_shape_table`) replays.

import assert from 'node:assert/strict';
import { before, test } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { formalizeDiagnostic, searchQuery } from '../../../js/agentic/repair_loop.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const FOURTEEN_LANGUAGES = [
  ['rust', 'error[E0308]: mismatched types\n --> src/main.rs:6:33', 'src/main.rs', 6, 'E0308', 'mismatched types'],
  ['typescript', "src/app.ts(4,7): error TS2322: Type 'string' is not assignable to type 'number'.", 'src/app.ts', 4, 'TS2322', "Type 'string' is not assignable to type 'number'."],
  ['javascript', '/work/index.js:3\nconsole.log(total);\n            ^\nerror: total is not defined', '/work/index.js', 3, null, 'total is not defined'],
  ['kotlin', 'Main.kt:3:5: error: unresolved reference: printn', 'Main.kt', 3, null, 'unresolved reference: printn'],
  ['scala', 'Main.scala:4: error: not found: value printn', 'Main.scala', 4, null, 'not found: value printn'],
  ['java', 'Main.java:5: error: cannot find symbol', 'Main.java', 5, null, 'cannot find symbol'],
  ['go', './main.go:7:2: undefined: fmt.Printn', './main.go', 7, null, 'undefined: fmt.Printn'],
  ['python', 'Traceback (most recent call last):\n  File "main.py", line 2, in <module>\n    print(1/0)\nZeroDivisionError: division by zero', 'main.py', 2, null, 'division by zero'],
  ['c', "main.c:4:5: error: implicit declaration of function 'printff'", 'main.c', 4, null, "implicit declaration of function 'printff'"],
  ['cpp', "main.cpp:6:3: error: 'cout' was not declared in this scope", 'main.cpp', 6, null, "'cout' was not declared in this scope"],
  ['csharp', "Program.cs(9,13): error CS0103: The name 'Consle' does not exist in the current context", 'Program.cs', 9, 'CS0103', "The name 'Consle' does not exist in the current context"],
  ['ruby', 'main.rb:2: syntax error, unexpected end-of-input', 'main.rb', 2, null, 'syntax error, unexpected end-of-input'],
  ['php', 'PHP Parse error: syntax error, unexpected end of file in /work/index.php on line 5', '/work/index.php', 5, null, 'syntax error, unexpected end of file'],
  ['swift', "main.swift:3:1: error: cannot find 'prin' in scope", 'main.swift', 3, null, "cannot find 'prin' in scope"],
];

test('every emitted language formalizes through the one shape table', () => {
  assert.equal(FOURTEEN_LANGUAGES.length, 14);
  for (const [language, raw, file, line, code, message] of FOURTEEN_LANGUAGES) {
    const read = formalizeDiagnostic(language, raw).map((d) => [d.file, d.line, d.code, d.message]);
    assert.deepEqual(read, [[file, line, code, message]], language);
  }
});

test('an inline php location does not swallow the diagnostic', () => {
  const read = formalizeDiagnostic(
    'php',
    'PHP Fatal error: Uncaught Error: Call to undefined function greet() in /work/index.php on line 3',
  ).map((d) => [d.file, d.line, d.message]);
  assert.deepEqual(read, [['/work/index.php', 3, 'Uncaught Error: Call to undefined function greet()']]);
});

test('the search query names the language and the exact code', () => {
  const [diagnostic] = formalizeDiagnostic('csharp', FOURTEEN_LANGUAGES[10][1]);
  assert.equal(searchQuery('csharp', diagnostic), 'csharp CS0103 The name Consle does not exist in the current context');
});
