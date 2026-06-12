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
- title-case, snake-case, kebab-case, camel-case, and pascal-case operations;
- URL, number, and email extraction operations;
- word, line, character, occurrence, and unique-word counting operations;
- punctuation removal, sentence-case, sort-words, strip-empty-lines, join-lines,
  reverse-lines, number-lines, indent-lines, outdent-lines, comment-lines, and
  uncomment-lines operations;
- Rust/browser-worker parity for the supported edit operations.

The regression tests are in
`tests/unit/specification/text_manipulation.rs`. The benchmark-family matrix is
in `tests/unit/specification/text_manipulation_benchmarks.rs`. The repository
local benchmark-source manifest is
`data/benchmarks/text-manipulation-suite.lino`.

## Benchmark Scope

PR #416 verifies the issue #408 edit behavior with:

- the original Russian follow-up replacement reproduction;
- 61 self-authored benchmark-family prompt-answer examples inspired by CoEdIT,
  EditEval, InstrEditBench/FineEdit, CodeEditorBench, CanItEdit, EDIT-Bench,
  HumanEvalPack/HumanEvalFix, and SWE-bench style editing tasks;
- a manifest-backed issue #408 text/code edit profile listing those 8 edit
  benchmark families plus 40 additional popular LLM benchmark sources;
- 30 deterministic prompt-answer variations per listed source, for 1,440
  generated checks total;
- per-source accounting that requires every source to pass 30/30 committed local
  checks, which exceeds the explicit repository-local 10% floor of 3 checks per
  source;
- an executable local ratchet,
  `issue_408_text_code_edit_profile_passes_local_ratchet`, that requires
  `minimum_pass_count = 1440` and reports the exact pass/fail count;
- the existing repository industry-suite slice, which remains separate from the
  issue #408 edit matrix.

The 1,440-case profile is the repository-local benchmark ratchet implemented for
this PR: every researched source has 30 committed local variations, the 10%
per-source floor is recorded as 3 checks, and the executable ratchet requires
all 30 checks for every source to pass. PR #416 therefore does not leave a
separate issue #408 benchmark task for another pull request; it intentionally
claims the repository-local edit benchmark profile and not an upstream
leaderboard score.

## Additional Benchmark Research

The raw research file records the same 48 sources as the executable manifest:
the 8 benchmark families already referenced by PR #416, 20 classic LLM
benchmarks, and 20 additional current/common LLM benchmarks used for language,
reasoning, coding, factuality, reading-comprehension, instruction-following,
chat, long-context, tool-use, multimodal, and evaluation-plus coverage:

`docs/case-studies/issue-408/raw-data/online-research.md`

The research keeps the implementation traceable: the repository now has a local
30-variation profile for every researched source, and the tests fail unless each
source independently passes every committed local variation.

## Requirement Mapping

- R293 covers the original issue #408 follow-up replacement behavior.
- R294 covers deterministic text/code edit operations and multilingual trigger
  parity.
- R295 covers the executable 48-source local profile with 30 variations per
  source and the 1,440-case pass-count ratchet.
- R296 covers the benchmark-source audit and documentation synchronization.
- R297 records the per-source 30/30 benchmark accounting and the fact that the
  issue #408 PR claim is repository-local, executable, and complete.
