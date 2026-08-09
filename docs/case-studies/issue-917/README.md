# Issue 917 Case Study

Issue [#917](https://github.com/link-assistant/formal-ai/issues/917) reported
that Formal AI's semantic translation layer could round-trip natural languages
and project a solved proof into programs, but could not treat a formal language
as an ordinary translation target. The implemented slice maps statements in
all five registered seed languages to first-order logic (FOL) and back
through one role-qualified semantic triple.

## 1. Collected Data

`raw-data/github/` contains snapshots of the issue, its comments, the prepared
PR, all three PR feedback channels, and related issues #526, #890, and #914.
`raw-data/online-research.md` records the official design references named by
the issue and the most recent repository prior art. Neither the issue nor its
comments contained an image attachment to download.

## 2. Requirements

The full matrix is in `requirements.md` and mirrored in root
`REQUIREMENTS.md` as R917-1 through R917-7. The acceptance criteria cover the
natural/formal round trip, seed-owned concrete syntaxes, extension of #526,
native/browser parity, traceability, and real Agent CLI authorship.

## 3. Reproduction And Root Cause

Before this change, the whole engine had no formal target catalog:

```text
request:  Translate `apple is a fruit` from English to FOL.
actual:   no natural/formal translation route
expected: P31(Q89, Q3314483)
inverse:  Translate `P31(Q89, Q3314483)` from FOL to Russian.
expected: яблоко это фрукт
```

The natural translator only projected atomic `MeaningId` values, while the
proof-to-program path introduced for #890 only recognized its interval-proof
syntax. No representation joined a relation and two entity roles into a
statement that a formal concrete syntax could render. Inverse lookup also
cannot select the first meaning carrying a Wikidata ID: the lexicon may ground
several distinct semantic roles at the same item.

## 4. Implemented Design

`SemanticStatement` carries a predicate, subject, and object meaning. The
formalizer resolves natural statements against the existing multilingual
lexicon and requires the appropriate `binary_relation` and `entity_anchor`
roles. `data/seed/formal-language-projections.lino` owns the FOL template,
aliases, natural word order, and canonical P31 relation surface. Rendering a
target interprets that catalog, so neither source/target pairs nor English word
order are compiled into the translation API.

The Rust-to-WASM worker interprets the same projection and Wikidata seed files.
The JavaScript worker only routes requests and passes seed text over the WASM
boundary. This is behavioral rather than visual UI work, so a real-browser
interaction regression is the appropriate evidence instead of before/after
screenshots. The output is a statement, not executable code, so E69 is not on
this path.

## 5. Verification

`every_seed_language_round_trips_through_a_seeded_formal_target` verifies all
five natural-to-FOL and all five FOL-to-natural projections retain
`statement:P31(Q89,Q3314483)`;
`every_seed_language_round_trips_through_first_order_logic` places the same
contract directly in issue #526's round-trip suite. The whole-engine test
checks user-facing intent, answer, and evidence in both directions.
`tests/e2e/tests/issue-917.spec.js`
repeats the complete matrix in Chromium against the local application.

`experiments/issue_917_agent_cli.sh` boots the real OpenAI-compatible server,
drives the installed Agent CLI, and compares the independently reviewed
formal-projection invariant leaf byte-for-byte. Its raw stream, server log,
session identifier, task, worktree status, and exact output are preserved under
`agent-cli-evidence/`. The captured session is
`ses_01c33a95effeAcU4AdF9Ec66Wr`.
