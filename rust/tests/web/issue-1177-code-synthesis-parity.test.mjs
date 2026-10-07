// Issue #1177 browser parity: regex and SQL composition.
//
// The worker twins in js/worker/formal_ai_worker_code_synthesis.js must answer
// the prompts rust/tests/unit/issue_1177_code_task_handlers.rs pins, at engine
// and handler level. The regex twin goes beyond Rust (R4): it compiles the
// composed pattern with RegExp and runs it on positive and negative examples.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

function handle(name, prompt) {
  const answer = worker[name](prompt, worker.normalizePrompt(prompt));
  assert.ok(answer, `${name} should answer: ${prompt}`);
  return answer.content;
}

const ZIP_PROMPT =
  "Write a regular expression that matches five digits optionally followed by a hyphen and four digits";
const SQL_PROMPT = "Write a SQL query that selects all users older than 30";

test("engine and handler compose the anchored postal pattern", async () => {
  const engine = await solve(ZIP_PROMPT);
  assert.equal(engine.intent, "regex_synthesis");
  assert.ok(engine.evidence.includes("response:regex_synthesis"));
  for (const answer of [engine.content, handle("handleRegexSynthesis", ZIP_PROMPT)]) {
    assert.ok(answer.includes("^\\d{5}(-\\d{4})?$"), answer);
    assert.ok(answer.includes("Verified structurally"), answer);
    assert.ok(answer.includes("no match was run against any input"), "the Rust template is kept verbatim");
  }
});

test("R4: the composed pattern is executed on positive and negative examples", () => {
  const answer = handle("handleRegexSynthesis", ZIP_PROMPT);
  assert.ok(
    answer.endsWith(
      "Verified by execution in the browser worker: the pattern was compiled with the JavaScript RegExp engine " +
        "and matched all 2 positive examples derived from the constraints (\"77777-7777\", \"77777\") and rejected " +
        "all 5 negative examples (\"7777-7777\", \"a7777-7777\", \"77777-777\", \"77777-a777\", \"77777-77777\").",
    ),
    answer,
  );
  const pattern = new RegExp("^\\d{5}(-\\d{4})?$");
  assert.ok(pattern.test("12345") && pattern.test("12345-6789") && !pattern.test("1234"));
});

test("R4: a pattern that disagrees with its examples is reported, not trusted", () => {
  const examples = worker.regexSynthesisExamples(
    worker.regexSynthesisScanMentions("five digits").classes,
    [],
    true,
  );
  const run = worker.regexSynthesisExecute("^\\d{4,}$", examples);
  assert.equal(run.status, "failed");
  assert.ok(run.failures.some((failure) => failure.startsWith("expected no match")), run.failures.join("; "));
});

test("handler composes the at-least repetition", () => {
  const answer = handle("handleRegexSynthesis", "Write a regular expression for three or more digits");
  assert.ok(answer.includes("\\d{3,}"), answer);
  assert.ok(answer.includes("(\"777\", \"77777\")"), "the at-least run is exercised with a longer positive");
});

test("engine and handler compose the filtered SELECT", async () => {
  const engine = await solve(SQL_PROMPT);
  assert.equal(engine.intent, "sql_synthesis");
  for (const answer of [engine.content, handle("handleSqlSynthesis", SQL_PROMPT)]) {
    assert.ok(answer.includes("SELECT"), answer);
    assert.ok(answer.includes("FROM users"), answer);
    assert.ok(answer.includes("age > 30"), answer);
    assert.ok(answer.includes("nothing was run"), answer);
  }
  assert.ok(engine.content.includes("    SELECT * FROM users WHERE age > 30;\n"), engine.content);
});

test("SQL composition reads order and limit clauses", () => {
  const answer = handle(
    "handleSqlSynthesis",
    "Write a SQL query that selects name from the employees table sorted by salary descending top 5",
  );
  assert.ok(answer.includes("SELECT name FROM employees ORDER BY salary DESC LIMIT 5;"), answer);
});

test("SQL composition groups an aggregate by the seeded grouping cue", () => {
  const counted = handle(
    "handleSqlSynthesis",
    "Write a SQL query to count users per country from the users table",
  );
  assert.ok(counted.includes("SELECT country, COUNT(*) FROM users GROUP BY country;"), counted);
  const averaged = handle(
    "handleSqlSynthesis",
    "Write a SQL query for the average salary for each department from the employees table",
  );
  assert.ok(
    averaged.includes("SELECT department, AVG(salary) FROM employees GROUP BY department;"),
    averaged,
  );
  const listed = handle(
    "handleSqlSynthesis",
    "Write a SQL query that selects name from the employees table for each row",
  );
  assert.ok(!listed.includes("GROUP BY"), "without an aggregate nothing is grouped: " + listed);
});

test("unrelated prompts are not claimed by the composers", () => {
  for (const prompt of ["Hello, how are you today?", "What is the capital of France?"]) {
    for (const name of ["handleRegexSynthesis", "handleSqlSynthesis"]) {
      assert.equal(worker[name](prompt, worker.normalizePrompt(prompt)), null, `${name} must not claim: ${prompt}`);
    }
  }
});

test("SQL composition reads HAVING after the grouping span", () => {
  const counted = handle(
    "handleSqlSynthesis",
    "Write a SQL query to count orders per customer from the orders table having at least 3 orders",
  );
  assert.ok(counted.includes("SELECT customer, COUNT(*) FROM orders GROUP BY customer HAVING COUNT(*) >= 3;"), counted);
  assert.ok(counted.includes("'having at least 3' -> HAVING COUNT(*) >= 3"), counted);
  const withCue = handle(
    "handleSqlSynthesis",
    "Write a SQL query to count users per country from the users table with more than 5 users",
  );
  assert.ok(withCue.includes("SELECT country, COUNT(*) FROM users GROUP BY country HAVING COUNT(*) > 5;"), withCue);
  const averaged = handle(
    "handleSqlSynthesis",
    "Write a SQL query for the average salary for each department from the employees table having average salary greater than 5000",
  );
  assert.ok(averaged.includes("GROUP BY department HAVING AVG(salary) > 5000;"), averaged);
  const beforeGrouping = handle(
    "handleSqlSynthesis",
    "Write a SQL query to count users with age greater than 30 per country from the users table",
  );
  assert.ok(beforeGrouping.includes("SELECT country, COUNT(*) FROM users WHERE age > 30 GROUP BY country;"), beforeGrouping);
});

test("SQL composition joins on a stated shared column", () => {
  assert.equal(handle("handleSqlSynthesis", "Write a SQL query that selects name from the users table joined with the orders table on user_id"), "Composed SQL:\n\n    SELECT name FROM users JOIN orders USING (user_id);\n\nClause-by-clause mapping:\n  - 'selects name' -> SELECT name\n  - 'users' -> FROM users\n  - 'joined with the orders table on user_id' -> JOIN orders USING (user_id)\nVerified by construction: each clause above maps to one constraint in the request, and the statement is a single SELECT in standard syntax.\nNot verified by execution: this project carries no SQL engine or parser dependency, so nothing was run — check the column names against your actual schema before running it.");
  const filtered = handle("handleSqlSynthesis", "Write a SQL query that selects name from the users table join orders using user_id where age greater than 30");
  assert.ok(filtered.includes("SELECT name FROM users JOIN orders USING (user_id) WHERE age > 30;"), filtered);
  const keyless = handle("handleSqlSynthesis", "Write a SQL query that selects name from the users table joined with orders");
  assert.ok(keyless.includes("SELECT name FROM users;"), "without a stated key no join is guessed: " + keyless);
  const russian = handle("handleSqlSynthesis", "Выбери всех users соедини с таблицей orders по user_id");
  assert.ok(russian.includes("SELECT * FROM users JOIN orders USING (user_id);"), russian);
});
