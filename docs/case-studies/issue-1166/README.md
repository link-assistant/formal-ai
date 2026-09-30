# Issue #1166 — The Request Formalized into Obligations

Issue [#1166](https://github.com/link-assistant/formal-ai/issues/1166) (E131):
routing and the executor decided what a request demands by matching surface
phrases, and the literal to print was collected once per mentioning clause.

## The doubled-output root cause (#1156)

The Kotlin Hello World issue body names the literal twice:

```
- The program must print exactly: "Hello, World!"
- Expected Output:
  "Hello, World!"
```

`explicit_stdout` in `rust/src/coding/program_contract.rs` collected every
`print_stdout`-meaning clause that carries a quoted span and joined them with
`\n` — two mentions, one join, and the program printed `Hello, World!` twice.
The executor had no notion that the two clauses *corefer*.

## The fix

`rust/src/intent_formalization/obligations.rs` formalizes the request first:
every enumerated clause becomes a ledger `ObligationNode` (`obligation_ledger`
supplies the node, the span, and the `Underivable` expectation — no new
ledger type), classified into `ObligationKind`s. Output literals are anchored
to their quoted value, and `coreference_pass` merges every mention of the
*same* value — across clauses, regardless of which clause they appear in —
into exactly one node. `unique_output_literal` therefore returns the value
once:

- three mentions in the canonical fixture → one output-literal node
  (`two_identical_output_clauses_produce_one_node`);
- two distinct literals → two nodes and *no* unique literal
  (`two_output_clauses_with_distinct_literals_produce_two_nodes`);
- a filename clause never merges with an output literal
  (`coreference_does_not_merge_filename_with_output_literal`).

## What the graph now catches that phrase matching dropped

The canonical body's "meaningful name like `test-hello-world.yml`", "add
clear comments / best practices", and "CI badge" clauses formalize to
`FileNaming`, `CodeStyle`, and `CiBadge` nodes — requirements the executor
previously never saw, which is how the workflow file came out as `run.yml`.

## Routing by meaning

`rust/src/solver_terminal.rs` now consults
`request_carries_work_obligations` in its leading-shell-token path: a quoted
output literal plus an authoring clause is a sentence about building
something — including in Russian, Hindi, Chinese, and Spanish, whose marker
words the English-only argument check of #1175 does not know — while
`git status` and `echo "Hello, World!"` remain commands.

## Multilingual and paraphrase parity

`rust/tests/fixtures/issue-1166/hello-world-kotlin-{en,ru,hi,zh,es}.txt` and
`-paraphrase-en.txt` carry the same seven-clause structure with per-language
enumeration cues (`First/Также/उसके बाद/接着/Después`) the ledger already
splits on; the parity tests assert the same node counts, the same obligation
kinds, and the same anchored literal. Classification reads the seed lexicon's
own multilingual lexemes first (`github actions 工作流`, `workflow de github
actions`, …); the fallback tables for style, naming, and badge vocabulary are
documented in-module as the interim until a seed meaning owns each one.

## Evidence

`cargo test --test unit issue_1166_obligation_routing` (fully offline).
