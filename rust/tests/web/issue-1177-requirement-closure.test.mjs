// Issue #1177 browser twins of the closed requirement gaps; the native pins
// are rust/tests/unit/web-engine-core/issue_1177_requirement_closure.rs.
//
// R1177-2: the composed pattern fills its execution line with the derived
// examples it was run against. R1177-5: every explained line is listed with
// its construct's grounding, or named as unexplained. R1177-7: stated examples
// become cases for a function outside every shape; with neither shape nor
// example the request is refused by name.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function handle(name, prompt) {
  const answer = worker[name](prompt, worker.normalizePrompt(prompt));
  assert.ok(answer, `${name} should answer: ${prompt}`);
  return answer.content;
}

test("R1177-2: the at-least pattern is run against its derived examples", () => {
  assert.equal(
    handle("handleRegexSynthesis", "Write a regular expression for three or more digits"),
    "Composed regular expression:\n\n```regex\n^\\d{3,}$\n```\n\nHow the constraints composed:\n  - '3 digits' -> `\\d{3,}` (the main run)\n  - anchored with ^ and $ so the whole string must match, not just a substring\n\nVerified structurally: groups are balanced, every repetition is well-formed, and every character class is closed.\nVerified by execution in the browser worker: the pattern was compiled with the JavaScript RegExp engine and matched all 2 positive examples derived from the constraints (\"777\", \"77777\") and rejected all 2 negative examples (\"77\", \"a77\").",
  );
});

test("R1177-5: every line is listed with its grounding or named unexplained", () => {
  const answer = handle(
    "handleCodeExplanation",
    "Explain this code:\n```python\nimport math\n# mean of the values\ndef average(xs):\n    total = 0\n    for x in xs:\n        total += x\n    while total > 100:\n        total -= 1\n    print(total)\n    yield total\n```",
  );
  assert.equal(
    answer,
    "A structural explanation, line by line. No code was executed.\n\n  - `import math` — loads math so the names it defines can be used below (https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement)\n  - `# mean of the values` — is a comment, ignored when the code runs; it tells the reader: mean of the values (https://docs.python.org/3.12/reference/lexical_analysis.html#comments)\n  - `def average(xs):` — defines a function named average which runs its indented body whenever it is called (https://docs.python.org/3.12/reference/compound_stmts.html#function-definitions)\n  - `total = 0` — binds the name total to the value on the right (https://docs.python.org/3.12/reference/simple_stmts.html#assignment-statements)\n  - `for x in xs:` — binds x to each element of xs in turn and runs the indented body once per element (https://docs.python.org/3.12/reference/compound_stmts.html#the-for-statement)\n  - `total += x` — updates total in place by applying the operator += with x (https://docs.python.org/3.12/reference/simple_stmts.html#augmented-assignment-statements)\n  - `while total > 100:` — repeats its body for as long as total > 100 stays true, checking before each pass (https://docs.python.org/3.12/reference/compound_stmts.html#the-while-statement)\n  - `total -= 1` — updates total in place by applying the operator -= with 1 (https://docs.python.org/3.12/reference/simple_stmts.html#augmented-assignment-statements)\n  - `print(total)` — calls the function named print with the arguments total (https://docs.python.org/3.12/reference/expressions.html#calls)\n  - `yield total` — no construct in the table matches this line, so it is left unexplained rather than guessed\nCoverage: 9 of 10 lines matched a construct; every line is listed above, each matched one with the reference that defines its construct.\nOverall: the function name `average` promises the arithmetic mean: the sum of the elements divided by their count; the canonical expression of that promise is `sum(xs) / len(xs)` (https://en.wikipedia.org/wiki/Arithmetic_mean).\n\nCost: a single pass over the input, so the work grows linearly (O(n)).\nMethod, stated honestly: each line was matched against the construct table in data/seed/meanings-code-structure-explanations.lino; nothing was run and nothing outside the table was guessed.",
  );
});

test("R1177-5: a JavaScript block reads its closing and keyword lines from seed rows", () => {
  const answer = handle(
    "handleCodeExplanation",
    "Explain this code:\n```js\nlet count = 0;\nwhile (count < 3) {\n  count += 1;\n}\n```",
  );
  assert.ok(answer.includes("  - `let count = 0;` — declares the variable count = 0, which later lines may reassign ("), answer);
  assert.ok(answer.includes("  - `while (count < 3) {` — repeats its body for as long as count < 3 stays true"), answer);
  assert.ok(answer.includes("  - `}` — closes the block opened above it (https://developer.mozilla.org/"), answer);
  assert.ok(answer.includes("Coverage: 4 of 4 lines matched a construct;"), answer);
});

test("R1177-7: stated examples become cases for a function outside every shape", () => {
  assert.equal(
    handle(
      "handleTestGeneration",
      "Write tests for `square(n)` where square(3) returns 9, square(-2) should return 4 and square(0) == 0",
    ),
    "Generated pytest suite (Python). NOT executed — executing generated tests is out of scope here (that is issue #1185's scope); review the cases and run them yourself with `pytest -q`.\n\n```python\ndef test_square_example_1():\n    assert square(3) == 9\ndef test_square_example_2():\n    assert square(-2) == 4\ndef test_square_example_3():\n    assert square(0) == 0\n```\n\nDerivation:   - 'square(3) returns 9' ->     assert square(3) == 9\n  - 'square(-2) should return 4' ->     assert square(-2) == 4\n  - 'square(0) == 0' ->     assert square(0) == 0\n",
  );
  assert.equal(
    handle("handleTestGeneration", "Напиши тесты для `is_prime(n)`: is_prime(7) возвращает True, is_prime(8) возвращает False"),
    "Generated pytest suite (Python). NOT executed — executing generated tests is out of scope here (that is issue #1185's scope); review the cases and run them yourself with `pytest -q`.\n\n```python\ndef test_is_prime_example_1():\n    assert is_prime(7) is True\ndef test_is_prime_example_2():\n    assert is_prime(8) is False\n```\n\nDerivation:   - 'is_prime(7) возвращает True' ->     assert is_prime(7) is True\n  - 'is_prime(8) возвращает False' ->     assert is_prime(8) is False\n",
  );
});

test("R1177-7: a word that is not a literal is not read as an expected value", () => {
  const answer = handle("handleTestGeneration", "Write tests for `f(x)`: f(x) is used often, f(2) is 4.5");
  assert.ok(answer.includes("def test_f_example_1():\n    assert f(2) == 4.5\n```"), answer);
  assert.ok(!answer.includes("used"), answer);
});

test("R1177-7: an unknown shape with no stated example is refused by name", () => {
  assert.equal(
    handle("handleTestGeneration", "Write tests for `is_even(n)`"),
    "Recognized a request for tests of `is_even`, but its name matches no problem shape in the test-case table (palindrome, average, max) and the request states no example with its expected value, so I will not guess what it must return. State examples as a call followed by its expected value, such as `is_even(<input>) returns <expected>` with a number, a quoted string, a list, True, False or None, and each one becomes a test case.",
  );
});

test("R1177-9: a CSV table converts to JSON and is checked cell by cell", () => {
  assert.equal(handle("handleFormatConversion", "Convert this CSV to JSON:\n```csv\nname,quote\nAnn,\"says \"\"hi\"\", twice\"\nBob,ok\n```"), "Converted CSV to JSON. Data rows: 2; header columns: name, quote. CSV carries no types, so every value stays a JSON string rather than a guessed number or boolean.\n\n```json\n[\n  {\n    \"name\": \"Ann\",\n    \"quote\": \"says \\\"hi\\\", twice\"\n  },\n  {\n    \"name\": \"Bob\",\n    \"quote\": \"ok\"\n  }\n]\n```\nRound-trip check: parsing the emitted JSON back gave every row the same value under every header column as the CSV it came from.");
  assert.equal(handle("handleFormatConversion", "Convert this CSV to JSON: name,age\nAnn"), "I will not guess a conversion. The text is outside the supported CSV subset (a header row of unique, non-empty names, then at least one row with the same number of comma-separated fields, double-quoted fields with doubled quotes inside), so I will not guess a conversion.");
});
