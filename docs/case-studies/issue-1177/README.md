# Issue #1177 Case Study: Nine Code Tasks, Derived Instead of Described

Issue [#1177](https://github.com/link-assistant/formal-ai/issues/1177) (E142,
part of the #1183 umbrella). Verified on `main` at `d209aac64`; fixed on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

Nine kinds of everyday programming requests, one shared failure shape — the
engine had no handler for any of them, so each prompt fell through every
route to the unknown-reasoning fallback of #1173, which describes a search
plan instead of deriving anything:

1. **"What's wrong with `def average(xs): return sum(xs) / len(xs) - 1`?"**
   — no debugging existed; the defect went unnamed.
2. **"Write a regular expression that matches five digits optionally
   followed by a hyphen and four digits"** — no regex composition existed.
3. **"Write a SQL query that selects all users older than 30"** — no SQL
   composition existed.
4. **"find .log files larger than 10 MB under /var"** — no shell-command
   composition existed.
5. **"Explain this code: `def average(xs): return sum(xs) / len(xs)`"** —
   no structural explanation existed.
6. **"Review this code: … `except:` …"** — no rule-based review existed.
7. **"Write tests for `is_palindrome(s)` ignoring case, spaces, and
   punctuation"** — no test generation existed.
8. **"Refactor this promise chain with async/await"** — no structural
   refactoring existed.
9. **"Convert this JSON to YAML"** — no format conversion existed.

All nine are derivations over data the prompt itself carries (code, counts,
constraints, documents). None of them needs execution, a network, or a
guess — which is exactly why a symbolic engine should be able to answer
them, and why a search-shaped fallback answer is the wrong shape entirely.

## Root cause (verified on `main` d209aac64)

No handler claimed any of these families: the dispatch chain had no
code-task entries, `rust/src/solver_handlers/` had no debugging, regex,
SQL, shell-composition, explanation, review, test-generation, refactoring
or format-conversion module, and the seed files held no cue vocabulary for
them. Every prompt reached #1173's fallback paragraph.

## The change

| File | Change |
| --- | --- |
| `rust/src/solver_handlers/code_debugging.rs` | quotient-shift defect scan grounded in the name-promise (`function_intent`) table |
| `rust/src/solver_handlers/regex_synthesis.rs` | pattern composition from seed count/class/separator/optional/anchor maps + structural verification |
| `rust/src/solver_handlers/sql_synthesis.rs` | single-SELECT composition with per-clause request mapping |
| `rust/src/solver_handlers/shell_command_compose.rs` | `find` composition with `-name`/`-size` from seed maps, manual cited |
| `rust/src/solver_handlers/code_explanation.rs` | line-by-line construct table with grounding URLs + name-promise summary |
| `rust/src/solver_handlers/code_review.rs` | five seed rules (Python + JavaScript), each with severity, advice and source URL |
| `rust/src/solver_handlers/test_generation.rs` | pytest suites for seed problem shapes, `normalize()` helper on cue |
| `rust/src/solver_handlers/code_refactoring.rs` | promise chain → `async`/`await`, handler bodies verbatim |
| `rust/src/solver_handlers/format_conversion.rs` | JSON↔YAML over a subset parser with a round-trip value check |
| `data/seed/code-task-cues.lino` | recognition cues and word maps for all nine families (en + ru) |
| `data/seed/code-review-rules.lino` | the five review rules with sources |
| `data/seed/multilingual-responses.lino` | the answer templates (en), every one carrying its honesty marker |
| `data/seed/meanings-code-task-templates.lino` | meanings records defining every new intent key, scope, role and package token |

## What a user sees now

Each probe is answered with the derived artifact and its honesty marker:
the debugging answer quotes line 1, names `- 1` as the shift, shows both
fixes and the property `average` promises with its source; the regex answer
is `^\d{5}(-\d{4})?$`, "Verified structurally", "No match was run"; the SQL
answer is `SELECT * FROM users WHERE age > 30;` with a clause-by-clause
mapping and "nothing was run"; the find answer is
`find /var -name '*.log' -size +10M` with the manual link and "Not
executed"; the explanation names the function, its constructs and its
promise; the review quotes `except:` and cites docs.python.org; the test
answer is a pytest suite over the shape's stated cases with "NOT executed";
the refactoring answer is a fenced `async function run()` with every
handler body verbatim; the conversion answer round-trips the document and
states the value check. Unrelated prompts ("What is the capital of
France?") are claimed by none of the nine handlers.

## Honest boundaries

- The handlers are delivered **unwired**: `try_dispatch_entries` and the
  lib re-exports are the umbrella integration step. The engine-level tests
  in `rust/tests/unit/issue_1177_code_task_handlers.rs` pass once that
  wiring lands; the handler-level tests pin the behavior now.
- One transformation per family: the debugging scan knows the
  quotient-shift defect; refactoring knows promise chains; shell
  composition knows `find`; test generation knows three problem shapes;
  format conversion knows a YAML subset. Anything else is refused by name —
  a named refusal is an answer, a guessed one is not.
- All structural checks are by construction; nothing is executed, and every
  template says so.
- Response templates are English-only; browser-worker and TypeScript roots
  are not delivered (the Rust handler is the reference implementation).
