//! Issue #1177: the code-task family — nine solver handlers for debugging,
//! regex synthesis, SQL synthesis, shell-command composition, code
//! explanation, code review, test generation, code refactoring, and
//! JSON/YAML format conversion.
//!
//! Every handler is structural: no code, regex, query, command, or generated
//! test is ever executed, and every answer template states that honestly.
//! The tests pin the contract from two angles:
//! - engine level: `UniversalSolver` answers each cued request with the
//!   derived artifact (these run once the dispatch chain wires the handlers),
//! - handler level: each `handle_*` function directly — the composition, the
//!   honesty marker, and the refusal to claim unrelated prompts.
//!
//! The solver runs offline so no test touches the network; prompts are
//! normalized with `web_engine_core::normalize_prompt`, the public twin of
//! the engine's normalizer.

use formal_ai::event_log::EventLog;
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{SolverConfig, UniversalSolver};

/// The answer a hermetic (offline) solver gives to `prompt`.
fn solved(prompt: &str) -> String {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    solver.solve(prompt).answer
}

/// The fenced block with the given tag, extracted from an answer body.
fn fenced(answer: &str, tag: &str) -> String {
    let open = ["```", tag, "\n"].concat();
    let start = answer
        .find(open.as_str())
        .unwrap_or_else(|| panic!("answer should carry a `{tag}` fence"))
        + open.len();
    let end = start
        + answer[start..]
            .find("```")
            .unwrap_or_else(|| panic!("the `{tag}` fence should be closed"));
    answer[start..end].to_owned()
}

// ---------------------------------------------------------------------------
// Engine level: the cued requests are answered with the derived artifacts.
// ---------------------------------------------------------------------------

#[test]
fn engine_answers_code_debugging_request() {
    let answer = solved("What's wrong with `def average(xs): return sum(xs) / len(xs) - 1`?");
    assert!(answer.contains("- 1"), "the shift must be named: {answer}");
    assert!(
        answer.contains("average"),
        "the function must be named: {answer}"
    );
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn engine_answers_regex_synthesis_request() {
    let answer = solved(
        "Write a regular expression that matches five digits optionally followed by a hyphen and four digits",
    );
    assert!(
        answer.contains("^\\d{5}(-\\d{4})?$"),
        "the composed pattern must be anchored: {answer}"
    );
    assert!(answer.contains("Verified structurally"), "{answer}");
}

#[test]
fn engine_answers_sql_synthesis_request() {
    let answer = solved("Write a SQL query that selects all users older than 30");
    assert!(answer.contains("SELECT"), "{answer}");
    assert!(answer.contains("FROM users"), "{answer}");
    assert!(answer.contains("age > 30"), "{answer}");
    assert!(answer.contains("nothing was run"), "{answer}");
}

#[test]
fn engine_answers_shell_compose_request() {
    let answer = solved("find .log files larger than 10 MB under /var");
    assert!(answer.contains("find /var"), "{answer}");
    assert!(answer.contains("-name '*.log'"), "{answer}");
    assert!(answer.contains("-size +10M"), "{answer}");
    assert!(answer.contains("Not executed"), "{answer}");
}

#[test]
fn engine_answers_code_explanation_request() {
    let answer = solved("Explain this code: `def average(xs): return sum(xs) / len(xs)`");
    assert!(answer.contains("function"), "{answer}");
    assert!(answer.contains("average"), "{answer}");
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn engine_answers_code_review_request() {
    let prompt = "Review this code:\n```python\ndef load(path):\n    try:\n        f = open(path)\n    except:\n        pass\n```";
    let answer = solved(prompt);
    assert!(
        answer.contains("except:"),
        "the offending line is quoted: {answer}"
    );
    assert!(answer.contains("docs.python.org"), "{answer}");
}

#[test]
fn engine_answers_test_generation_request() {
    let answer =
        solved("Write tests for `is_palindrome(s)` ignoring case, spaces, and punctuation");
    assert!(answer.contains("def test_"), "{answer}");
    assert!(answer.contains("RaceCar"), "{answer}");
    assert!(answer.contains("is_palindrome(normalize("), "{answer}");
    assert!(answer.contains("NOT executed"), "{answer}");
}

#[test]
fn engine_answers_code_refactoring_request() {
    let prompt = "Refactor this promise chain with async/await:\n```javascript\nfetch(url)\n  .then(r => r.json())\n  .then(d => render(d))\n  .catch(e => log(e));\n```";
    let answer = solved(prompt);
    let code = fenced(&answer, "javascript");
    assert!(code.contains("async function run()"), "{answer}");
    assert!(code.contains("await fetch(url)"), "{answer}");
    assert!(code.contains("catch (e)"), "{answer}");
}

#[test]
fn engine_answers_format_conversion_request() {
    let json_text = "{\"name\": \"formal-ai\", \"tags\": [\"rust\", \"lino\"], \"counts\": {\"lines\": 42, \"passed\": 41}}";
    let to_yaml = ["Convert this JSON to YAML:\n```json\n", json_text, "\n```"].concat();
    let yaml_answer = solved(&to_yaml);
    let yaml_text = fenced(&yaml_answer, "yaml");
    assert!(yaml_text.contains("name: formal-ai"), "{yaml_answer}");
    assert!(yaml_text.contains("lines: 42"), "{yaml_answer}");

    // Feeding the emitted YAML back must reproduce the same values as JSON.
    let to_json = ["Convert this YAML to JSON:\n```yaml\n", &yaml_text, "\n```"].concat();
    let json_answer = solved(&to_json);
    let emitted = fenced(&json_answer, "json");
    let expected: serde_json::Value = serde_json::from_str(json_text).expect("probe json parses");
    let actual: serde_json::Value =
        serde_json::from_str(emitted.as_str()).expect("emitted json parses");
    assert_eq!(actual, expected, "{json_answer}");
}

// ---------------------------------------------------------------------------
// Handler level: each `handle_*` function directly.
// ---------------------------------------------------------------------------

/// Run one handler over a prompt and return its answer body.
macro_rules! answer_of {
    ($handler:path, $prompt:expr) => {{
        let prompt: &str = $prompt;
        let normalized = normalize_prompt(prompt);
        let mut log = EventLog::new();
        let answer = $handler(prompt, &normalized, &mut log)
            .unwrap_or_else(|| panic!("{} should answer this request", stringify!($handler)));
        answer.answer
    }};
}

#[test]
fn handler_code_debugging_reports_shifted_quotient() {
    let answer = answer_of!(
        formal_ai::handle_code_debugging,
        "What's wrong with `def average(xs): return sum(xs) / len(xs) - 1`?"
    );
    assert!(answer.contains("- 1"), "the shift must be named: {answer}");
    assert!(
        answer.contains("sum(xs)"),
        "the fixed form must be shown: {answer}"
    );
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn handler_regex_synthesis_composes_anchored_pattern() {
    let answer = answer_of!(
        formal_ai::handle_regex_synthesis,
        "Write a regular expression that matches five digits optionally followed by a hyphen and four digits"
    );
    assert!(
        answer.contains("^\\d{5}(-\\d{4})?$"),
        "the composed pattern must be anchored: {answer}"
    );
    assert!(answer.contains("Verified structurally"), "{answer}");
    assert!(
        answer.contains("no match was run against any input"),
        "{answer}"
    );
}

#[test]
fn handler_regex_synthesis_composes_at_least_repetition() {
    let answer = answer_of!(
        formal_ai::handle_regex_synthesis,
        "Write a regular expression for three or more digits"
    );
    assert!(
        answer.contains("\\d{3,}"),
        "the at-least repetition must render: {answer}"
    );
}

#[test]
fn handler_sql_synthesis_composes_filtered_select() {
    let answer = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query that selects all users older than 30"
    );
    assert!(answer.contains("SELECT"), "{answer}");
    assert!(answer.contains("FROM users"), "{answer}");
    assert!(answer.contains("age > 30"), "{answer}");
    assert!(answer.contains("nothing was run"), "{answer}");
}

#[test]
fn handler_sql_synthesis_groups_an_aggregate_by_the_seeded_cue() {
    let counted = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query to count users per country from the users table"
    );
    assert!(
        counted.contains("SELECT country, COUNT(*) FROM users GROUP BY country;"),
        "{counted}"
    );
    let averaged = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query for the average salary for each department from the employees table"
    );
    assert!(
        averaged.contains("SELECT department, AVG(salary) FROM employees GROUP BY department;"),
        "{averaged}"
    );
    let listed = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query that selects name from the employees table for each row"
    );
    assert!(
        !listed.contains("GROUP BY"),
        "without an aggregate nothing is grouped: {listed}"
    );
}

#[test]
fn handler_sql_synthesis_filters_groups_with_having_after_the_grouping() {
    let answer = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query to count orders per customer from the orders table having at least 3 orders"
    );
    let expected = "Composed SQL:\n\n    SELECT customer, COUNT(*) FROM orders GROUP BY customer HAVING COUNT(*) >= 3;\n\nClause-by-clause mapping:\n  - 'count' -> SELECT customer, COUNT(*)\n  - 'orders' -> FROM orders\n  - 'per customer' -> GROUP BY customer\n  - 'having at least 3' -> HAVING COUNT(*) >= 3\nVerified by construction: each clause above maps to one constraint in the request, and the statement is a single SELECT in standard syntax.\nNot verified by execution: this project carries no SQL engine or parser dependency, so nothing was run \u{2014} check the column names against your actual schema before running it.";
    assert_eq!(answer, expected);
    let with_cue = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query to count users per country from the users table with more than 5 users"
    );
    assert!(
        with_cue
            .contains("SELECT country, COUNT(*) FROM users GROUP BY country HAVING COUNT(*) > 5;"),
        "the threshold applies to the aggregate, not to a WHERE column: {with_cue}"
    );
    let before_grouping = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query to count users with age greater than 30 per country from the users table"
    );
    assert!(
        before_grouping
            .contains("SELECT country, COUNT(*) FROM users WHERE age > 30 GROUP BY country;"),
        "a comparison stated before the grouping stays a WHERE filter: {before_grouping}"
    );
}

#[test]
fn handler_code_debugging_reports_an_assignment_inside_a_condition() {
    let answer = answer_of!(
        formal_ai::handle_code_debugging,
        "Find the bug in this code:\n```js\nfunction check(x) {\n  if (x = 5) {\n    return true;\n  }\n  return false;\n}\n```"
    );
    let expected = "An assignment where a comparison was meant.\n\nDetected defect (code line 2): `if (x = 5) {`\nThe condition assigns with a single `=`, so it tests the assigned value instead of comparing: the branch runs whenever that value is truthy, and the variable is overwritten.\n\nFix \u{2014} compare instead of assigning (in JavaScript, `===` is the strict comparison):\n    if (x == 5) {\n\nMethod, stated honestly: a structural scan of each condition that opens with a head from the condition table (`assignment_in_condition` in data/seed/code-task-cues.lino) for a lone `=` at the top level of its parentheses, outside quoted text. No code was executed and no test was run.";
    assert_eq!(answer, expected);
    let nested = answer_of!(
        formal_ai::handle_code_debugging,
        "Debug this:\n```js\nfunction readAll(r) {\n  let line;\n  while ((line = r.read()) != null) { console.log(line); }\n}\n```"
    );
    assert!(
        nested.starts_with("No recognized defect pattern."),
        "a nested assignment is deliberate: {nested}"
    );
    let quoted = answer_of!(
        formal_ai::handle_code_debugging,
        "Find the bug in this code:\n```js\nfunction f(s) {\n  if (s == \"a=b\") { return 1; }\n  return 0;\n}\n```"
    );
    assert!(
        quoted.starts_with("No recognized defect pattern."),
        "an = inside quoted text is not code: {quoted}"
    );
}

#[test]
fn handler_sql_synthesis_joins_on_a_stated_shared_column() {
    let answer = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query that selects name from the users table joined with the orders table on user_id"
    );
    let expected = "Composed SQL:\n\n    SELECT name FROM users JOIN orders USING (user_id);\n\nClause-by-clause mapping:\n  - 'selects name' -> SELECT name\n  - 'users' -> FROM users\n  - 'joined with the orders table on user_id' -> JOIN orders USING (user_id)\nVerified by construction: each clause above maps to one constraint in the request, and the statement is a single SELECT in standard syntax.\nNot verified by execution: this project carries no SQL engine or parser dependency, so nothing was run \u{2014} check the column names against your actual schema before running it.";
    assert_eq!(answer, expected);
    let filtered = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query that selects name from the users table join orders using user_id where age greater than 30"
    );
    assert!(
        filtered.contains("SELECT name FROM users JOIN orders USING (user_id) WHERE age > 30;"),
        "{filtered}"
    );
    let keyless = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query that selects name from the users table joined with orders"
    );
    assert!(
        keyless.contains("SELECT name FROM users;"),
        "without a stated key no join is guessed: {keyless}"
    );
    let russian = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Выбери всех users соедини с таблицей orders по user_id"
    );
    assert!(
        russian.contains("SELECT * FROM users JOIN orders USING (user_id);"),
        "{russian}"
    );
}

#[test]
fn handler_code_debugging_reports_a_loop_bound_past_the_end() {
    let answer = answer_of!(
        formal_ai::handle_code_debugging,
        "Find the bug in this code:\n```python\ndef total(xs):\n    s = 0\n    for i in range(len(xs) + 1):\n        s += xs[i]\n    return s\n```"
    );
    assert_eq!(
        answer,
        "A loop bound past the end of `xs`.\n\nDetected defect (code line 3): `for i in range(len(xs) + 1):`\nThe bound `range(len(xs) + 1)` lets the index reach the length of `xs`, one past its last valid position, so the final pass reads `xs[…]` out of range.\n\nFix — stop one position earlier:\n    for i in range(len(xs)):\n\nMethod, stated honestly: a structural scan against the index-bound table (`index_bound_past_end` in data/seed/code-task-cues.lino), confirmed by an index into `xs` in the same code. No code was executed and no test was run."
    );
    let bounded = answer_of!(
        formal_ai::handle_code_debugging,
        "Debug this:\n```js\nfunction last(arr) {\n  let out = 0;\n  for (let i = 0; i <= arr.length; i++) { out = arr[i]; }\n  return out;\n}\n```"
    );
    assert!(
        bounded.contains("    for (let i = 0; i < arr.length; i++) { out = arr[i]; }\n"),
        "{bounded}"
    );
}

#[test]
fn handler_shell_compose_slices_lines_and_searches_content() {
    let sliced = answer_of!(
        formal_ai::handle_shell_command_compose,
        "show the last 20 lines of app.log"
    );
    assert!(sliced.contains("    tail -n 20 app.log\n"), "{sliced}");
    let searched = answer_of!(
        formal_ai::handle_shell_command_compose,
        "search for TODO in files under /src ignoring case"
    );
    assert!(
        searched.contains("    grep -r -i 'TODO' /src\n"),
        "{searched}"
    );
}

#[test]
fn handler_shell_compose_composes_a_literal_sed_substitution() {
    let answer = answer_of!(
        formal_ai::handle_shell_command_compose,
        "Replace 1.0 with 2.0 in version.txt"
    );
    let expected = "Composed shell command:\n\n    sed -i 's/1\\.0/2.0/g' version.txt\n\nFlags, from the sed manual:\n  - `-i` \u{2014} edit the file in place (a GNU extension); without it sed prints the edited text and leaves the file unchanged\n  - `s/regexp/replacement/g` \u{2014} replace each match of the basic regular expression with the replacement; the trailing g replaces every match on a line, not only the first\nManual: https://www.gnu.org/software/sed/manual/sed.html\n\nDerivation:   - 'Replace 1.0 with 2.0' -> s/1\\.0/2.0/g\n  - 'version.txt' -> version.txt\n\nNot executed: this is a composed suggestion \u{2014} run it yourself and inspect the output.";
    assert_eq!(answer, expected);
    let russian = solved("Замени foo на bar в файле config.txt");
    assert!(
        russian.contains("    sed -i 's/foo/bar/g' config.txt\n"),
        "{russian}"
    );
    let sentence = "Replace cat with dog in this sentence";
    let claimed = formal_ai::handle_shell_command_compose(
        sentence,
        &normalize_prompt(sentence),
        &mut EventLog::new(),
    );
    assert!(
        claimed.is_none(),
        "without a file argument the request is not a shell task"
    );
}

#[test]
fn handler_shell_compose_builds_find_command() {
    let answer = answer_of!(
        formal_ai::handle_shell_command_compose,
        "find .log files larger than 10 MB under /var"
    );
    assert!(answer.contains("find /var"), "{answer}");
    assert!(answer.contains("-name '*.log'"), "{answer}");
    assert!(answer.contains("-size +10M"), "{answer}");
    assert!(answer.contains("Not executed"), "{answer}");
}

/// R1017: a relative path ("data/meta") is the search root, not ".".
#[test]
fn handler_shell_compose_reads_a_relative_root() {
    let answer = answer_of!(
        formal_ai::handle_shell_command_compose,
        "find .lino files under data/meta"
    );
    assert!(answer.contains("find data/meta -name '*.lino'"), "{answer}");
}

/// "files ending in .lino" names the `-name` suffix, and the "write a shell
/// command that finds ..." framing reads the third-person action cue.
#[test]
fn handler_shell_compose_reads_an_ending_suffix() {
    let answer = answer_of!(
        formal_ai::handle_shell_command_compose,
        "Write a shell command that finds files ending in .lino under data/meta"
    );
    assert!(answer.contains("find data/meta -name '*.lino'"), "{answer}");
    assert!(!answer.contains("wc -l"), "{answer}");
    assert!(answer.contains("Not executed"), "{answer}");
}

/// A seeded counting cue pipes the find into `wc -l`, explained from wc's
/// manual record.
#[test]
fn handler_shell_compose_pipes_a_count_into_wc() {
    for prompt in [
        "count .lino files under data/meta",
        "Write a shell command that counts files ending in .lino under data/meta",
    ] {
        let answer = answer_of!(formal_ai::handle_shell_command_compose, prompt);
        assert!(
            answer.contains("find data/meta -name '*.lino' | wc -l"),
            "{answer}"
        );
        assert!(
            answer.contains(
                "https://www.gnu.org/software/coreutils/manual/html_node/wc-invocation.html"
            ),
            "{answer}"
        );
        assert!(answer.contains("`-l`"), "{answer}");
        assert!(answer.contains("Not executed"), "{answer}");
    }
}

/// A requested function/program (a seeded `program_artifact` word) is
/// program synthesis, not a shell command.
#[test]
fn handler_shell_compose_leaves_program_requests() {
    let prompt = "Write a JavaScript function that counts the lines of a text";
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    assert!(
        formal_ai::handle_shell_command_compose(prompt, &normalized, &mut log).is_none(),
        "shell compose must not claim: {prompt}"
    );
}

/// The containment cue words are seed data, Russian forms included.
#[test]
fn handler_shell_compose_reads_a_russian_containment_cue() {
    let answer = answer_of!(
        formal_ai::handle_shell_command_compose,
        "найди файлы содержащие TODO в /src"
    );
    assert!(answer.contains("grep -r 'TODO' /src"), "{answer}");
}

#[test]
fn engine_answers_counting_shell_request() {
    let answer = solved("Write a shell command that counts files ending in .lino under data/meta");
    assert!(
        answer.contains("find data/meta -name '*.lino' | wc -l"),
        "{answer}"
    );
    assert!(answer.contains("Not executed"), "{answer}");
}

/// R1017: an inline quoted text payload under a seeded text operation is the
/// text handler's; "lines" alone is no filesystem operand for the composer.
#[test]
fn handler_shell_compose_defers_inline_text_operations() {
    let prompt = "Count the lines in this text: 'a\nb\nc'";
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    assert!(
        formal_ai::handle_shell_command_compose(prompt, &normalized, &mut log).is_none(),
        "shell compose must not claim: {prompt}"
    );
}

#[test]
fn handler_code_explanation_names_function_and_promise() {
    let answer = answer_of!(
        formal_ai::handle_code_explanation,
        "Explain this code: `def average(xs): return sum(xs) / len(xs)`"
    );
    assert!(answer.contains("function"), "{answer}");
    assert!(answer.contains("average"), "{answer}");
    assert!(answer.contains("sum(xs) / len(xs)"), "{answer}");
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn handler_code_review_reports_bare_except_with_source() {
    let prompt = "Review this code:\n```python\ndef load(path):\n    try:\n        f = open(path)\n    except:\n        pass\n```";
    let answer = answer_of!(formal_ai::handle_code_review, prompt);
    assert!(
        answer.contains("except:"),
        "the offending line is quoted: {answer}"
    );
    assert!(answer.contains("docs.python.org"), "{answer}");
    assert!(answer.contains("no linter was run"), "{answer}");
}

#[test]
fn handler_test_generation_builds_palindrome_suite() {
    let answer = answer_of!(
        formal_ai::handle_test_generation,
        "Write tests for `is_palindrome(s)` ignoring case, spaces, and punctuation"
    );
    assert!(answer.contains("def test_"), "{answer}");
    assert!(answer.contains("RaceCar"), "{answer}");
    assert!(answer.contains("never odd or even"), "{answer}");
    assert!(answer.contains("def normalize(value):"), "{answer}");
    assert!(answer.contains("is_palindrome(normalize("), "{answer}");
    assert!(answer.contains("NOT executed"), "{answer}");
}

#[test]
fn handler_code_refactoring_rewrites_promise_chain() {
    let prompt = "Refactor this promise chain with async/await:\n```javascript\nfetch(url)\n  .then(r => r.json())\n  .then(d => render(d))\n  .catch(e => log(e));\n```";
    let answer = answer_of!(formal_ai::handle_code_refactoring, prompt);
    let code = fenced(&answer, "javascript");
    assert!(code.contains("async function run()"), "{answer}");
    assert!(code.contains("await fetch(url)"), "{answer}");
    assert!(code.contains("await r.json()"), "{answer}");
    assert!(code.contains("await render(d)"), "{answer}");
    assert!(code.contains("catch (e)"), "{answer}");
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn handler_format_conversion_round_trips_json_and_yaml() {
    let json_text = "{\"name\": \"formal-ai\", \"tags\": [\"rust\", \"lino\"], \"counts\": {\"lines\": 42, \"passed\": 41}}";
    let to_yaml = ["Convert this JSON to YAML:\n```json\n", json_text, "\n```"].concat();
    let yaml_answer = answer_of!(formal_ai::handle_format_conversion, &to_yaml);
    let yaml_text = fenced(&yaml_answer, "yaml");
    assert!(yaml_text.contains("name: formal-ai"), "{yaml_answer}");
    assert!(yaml_text.contains("- rust"), "{yaml_answer}");
    assert!(yaml_text.contains("lines: 42"), "{yaml_answer}");

    let to_json = ["Convert this YAML to JSON:\n```yaml\n", &yaml_text, "\n```"].concat();
    let json_answer = answer_of!(formal_ai::handle_format_conversion, &to_json);
    let emitted = fenced(&json_answer, "json");
    let expected: serde_json::Value = serde_json::from_str(json_text).expect("probe json parses");
    let actual: serde_json::Value =
        serde_json::from_str(emitted.as_str()).expect("emitted json parses");
    assert_eq!(actual, expected, "{json_answer}");
}

#[test]
fn unrelated_prompts_are_not_claimed_by_any_code_task_handler() {
    type Handler = fn(&str, &str, &mut EventLog) -> Option<formal_ai::engine::SymbolicAnswer>;

    let unrelated = [
        "Hello, how are you today?",
        "What is the capital of France?",
    ];
    let handlers: [(Handler, &str); 9] = [
        (formal_ai::handle_code_debugging, "debugging"),
        (formal_ai::handle_regex_synthesis, "regex"),
        (formal_ai::handle_sql_synthesis, "sql"),
        (formal_ai::handle_shell_command_compose, "shell"),
        (formal_ai::handle_code_explanation, "explanation"),
        (formal_ai::handle_code_review, "review"),
        (formal_ai::handle_test_generation, "test generation"),
        (formal_ai::handle_code_refactoring, "refactoring"),
        (formal_ai::handle_format_conversion, "conversion"),
    ];
    for prompt in unrelated {
        let normalized = normalize_prompt(prompt);
        for (handler, name) in handlers {
            let mut log = EventLog::new();
            assert!(
                handler(prompt, &normalized, &mut log).is_none(),
                "{name} must not claim: {prompt}"
            );
        }
    }
}
