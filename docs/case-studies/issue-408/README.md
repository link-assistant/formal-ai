# Issue 408 Case Study

Issue: https://github.com/link-assistant/formal-ai/issues/408
Pull request: https://github.com/link-assistant/formal-ai/pull/416
Updated: 2026-06-12

## Reported Failure

Issue #408 reported a Russian multi-turn editing dialog. The assistant first
generated a Rust hello-world program. The user then asked to replace "Hello
World" with "Bye world". The solver should have edited the previously generated
code artifact, but instead fell through to a generic non-neural fallback.

## Implemented Fix

PR #416 routes deterministic text and code edits through the same symbolic edit
surface used for normal text manipulation. The implemented behavior covers:

- follow-up replacements that refer to the previous assistant code block;
- direct text replacement requests where the source text is embedded in the
  current prompt;
- multilingual operation triggers through `data/seed/operation-vocabulary.lino`;
- punctuation-tolerant replacement across generated code and plain text;
- remove, append, prepend, trim-whitespace, and normalize-whitespace operations;
- Rust/browser-worker parity for the supported edit operations.

The regression tests are in
`tests/unit/specification/text_manipulation.rs`. The benchmark-family matrix is
in `tests/unit/specification/text_manipulation_benchmarks.rs`.

## Benchmark Scope

PR #416 verifies the issue #408 edit behavior with:

- the original Russian follow-up replacement reproduction;
- 50 self-authored benchmark-family prompt-answer examples inspired by CoEdIT,
  EditEval, InstrEditBench/FineEdit, CodeEditorBench, CanItEdit, EDIT-Bench,
  HumanEvalPack/HumanEvalFix, and SWE-bench style editing tasks;
- the existing repository industry-suite slice, which remains separate from the
  issue #408 edit matrix.

This PR does not claim 10% of every upstream benchmark. A full upstream
benchmark pass claim would require the repository to import or reference the
full upstream dataset snapshot or a documented sample, preserve license and
provenance metadata, implement the benchmark's runner and scoring contract, fit
that execution into CI, and ratchet the pass count against the imported task
records. That infrastructure is not equivalent to the self-authored regression
examples used here.

## Additional Benchmark Research

The raw research file records the 8 benchmark families already referenced by
PR #416 and 20 additional popular LLM benchmarks that are commonly used for
language, reasoning, coding, factuality, reading-comprehension, and instruction
following evaluation:

`docs/case-studies/issue-408/raw-data/online-research.md`

The research keeps the current implementation honest: the repository may use
small, reviewable slices and benchmark-derived examples, but it must not say it
passes 10% of a full external benchmark unless the full scoring pipeline is
present and checked.

## Requirement Mapping

- R293 covers the original issue #408 follow-up replacement behavior.
- R294 covers deterministic text/code edit operations and multilingual trigger
  parity.
- R295 covers the 50 self-authored benchmark-family regression examples.
- R296 covers the benchmark-source audit and documentation synchronization.
- R297 records the boundary for any future claim about passing 10% of every
  upstream benchmark.
