# Issue 103 Case Study: Expand Test Cases and Document Evolving Architecture

## Summary

Issue [#103](https://github.com/link-assistant/formal-ai/issues/103) asks
formal-ai to grow each existing test case into **5–10 most probable input
variations** and translate every case across the four currently supported
languages — English, Russian, Hindi, and Chinese — so that every behavior is
exercised across roughly _category × variation × language_ matrices instead of
single-shot prompts. The issue also asks the project to:

- compare the existing prompt catalog against the test surfaces of competitor
  AI models and agentic CLI tools, and adopt the most frequent prompts that we
  do not already cover;
- generalize the test-case scaffolding so the matrix can grow without bloating
  Rust source files;
- create a detailed `ARCHITECTURE.md` that captures the evolving design
  (Links-Notation-first context, Wikidata-anchored formalization, neural-style
  temperature controls over candidate interpretations, doublets-rs/doublets-web
  persistence, transformation rules stored as data or compiled code, and
  formalization-driven translation between natural and programming languages);
- keep `VISION.md` and `REQUIREMENTS.md` in lockstep with the new architecture
  description.

The implementation in this PR follows that brief end to end. It introduces
parametrized prompt matrices for every conversational surface, ports the most
frequent competitor prompts onto formal-ai's deterministic core, generalizes
the test data via shared helper modules, and adds an `ARCHITECTURE.md` that
captures the evolving design with cross-references back into vision and
requirements.

## Collected Data

Fresh evidence is preserved under `raw-data/`:

- `issue-103.json` and `issue-103-comments.json` — the source issue body and
  comments captured on 2026-05-17.
- `pr-104.json`, `pr-104-conversation-comments.json`,
  `pr-104-review-comments.json`, `pr-104-reviews.json` — the prepared PR state
  on the same day.
- `recent-merged-prs.json` — neighboring merged PRs (issue #82, the v0.54.0
  release) used to mirror commit-message and PR-body conventions.
- `competitor-test-research.md` — the full competitor-test research document
  used to drive the prompt-variation matrix. It catalogs:
  1. Agentic CLI tool test surfaces (Claude Code, Aider, Codex, Continue,
     Cursor, GitHub Copilot CLI).
  2. LLM benchmarks relevant to a symbolic chat assistant (MMLU, HellaSwag,
     GLUE/SuperGLUE, BIG-bench, HumanEval, MBPP, GSM8K, TruthfulQA, Aya,
     XCOPA, Belebele, XNLI, FLORES-200, MGSM, TyDi QA, XQuAD).
  3. Conversational AI benchmarks (Chatbot Arena, MT-Bench, AlpacaEval,
     WildBench, Vicuna-80).
  4. The four-language coverage we already commit to: EN, RU, HI, ZH.
  5. The top-10 conversational prompt categories distilled from public
     leaderboards.
  6. Rust testing tools (`rstest`, `proptest`, `insta`, `test-case`,
     `datatest-stable`, `assert_cmd`) and patterns worth reusing
     (table-driven data files, golden snapshots, matrix runners).

## Requirements

Issue #103 extends the requirement matrix to R129+ in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md). Each row below is the canonical
mapping from issue text to PR-104 work.

| ID | Requirement | Source | Solution in this PR |
| --- | --- | --- | --- |
| R129 | For every existing test case, exercise 5–10 most probable input/output variations. | Issue body | Add `tests/unit/specification/prompt_variations.rs` and expand `chat_surface.rs`, `code_generation.rs`, and `multilingual.rs` with `for (prompt, _) in [...]` matrices that cover greeting, identity, capabilities, concept lookup, code-generation, idioms, transliteration, clarification, and math intents. |
| R130 | Translate each test case into English, Russian, Hindi, and Chinese. | Issue body | Each matrix block iterates over `(language, prompts)` tuples covering all four languages. Helper assertions confirm the right `language:*` evidence link is emitted on every prompt. |
| R131 | Compare formal-ai's tests against competitor AI models and agentic CLI tools, and adopt the most frequent / probable prompts. | Issue body | `docs/case-studies/issue-103/raw-data/competitor-test-research.md` indexes Claude Code, Aider, Codex, Continue, Cursor, Copilot CLI, MT-Bench, AlpacaEval, and the multilingual benchmark family; the high-frequency categories (definitions, summarization-intent, brainstorming-intent, factual Q&A, refusal/safety, multi-turn coreference, roleplay) are implemented as active regression tests backed by deterministic solver handlers. |
| R132 | Generalize the test-case logic where possible. | Issue body | New `tests/unit/specification/prompt_variations.rs` introduces helpers `assert_intent_for_each`, `assert_language_for_each`, and `assert_answer_contains_for_each` plus four-language matrix builders so future categories can be added in one block instead of per-language test functions. |
| R133 | Add a detailed `ARCHITECTURE.md` describing the evolving architecture. | Issue body | New `ARCHITECTURE.md` at the repository root documents the full pipeline: context assembly → translation to Links Notation → Wikidata P/Q-ID formalization → temperature-style interpretation selection → clarifying-question vs guessing under `SolverConfig` → nested reasoning steps with tool integrations → growable doublets-rs/doublets-web memory → .lino backups → transformation rules in data / Rust / JS / natural language → formalization-driven translation. |
| R134 | Update `VISION.md` to reflect the architecture description from the issue (Wikidata P/Q-ID formalization, temperature-style interpretation selection, nested reasoning, growable doublet memory, transformation rules in data, on-demand compilation of natural-language skills, formalization-driven translation). | Issue body ("Double check our requirements and vision are updated…") | New **Formalization And Temperature** section in `VISION.md` plus updated **Computation Model**, **Data Is The Interface**, and **Reasoning Model** paragraphs explicitly call out the architecture additions. |
| R135 | Update `REQUIREMENTS.md` to expose R129+ alongside the existing matrix. | Issue body | The R129–R136 block is appended under a new **Issue #103 Test-Matrix and Architecture Requirements** heading. |
| R136 | Compile issue #103 evidence and case-study analysis under `docs/case-studies/issue-103/`. | Issue body | This directory contains the raw GitHub data, competitor research, and the case-study README you are reading. |

## Online And Upstream Research

The full research notes live in
[`raw-data/competitor-test-research.md`](raw-data/competitor-test-research.md).
The relevant findings for this PR:

1. **Agentic CLI tools do not ship conversational test suites.** Claude Code,
   Aider, Codex, Continue, Cursor, and GitHub Copilot CLI use code-editing or
   refactor-style benchmarks (SWE-Bench, HumanEval, MBPP, the Aider Polyglot
   leaderboard). They do not cover greetings, identity, idioms, or multilingual
   chat — meaning formal-ai's conversational matrix is filling a gap rather
   than chasing an industry SOTA score.
2. **Conversational benchmarks converge on ~10 categories.** MT-Bench
   (Writing / Roleplay / Reasoning / Math / Coding / Extraction / STEM /
   Humanities), AlpacaEval, WildBench, and Chatbot Arena's category tags all
   compress into the same ten conversational categories that formal-ai already
   targets. The gaps are: roleplay (optional), refusal/safety, multi-turn
   coreference, summarization-intent, and brainstorming-intent. These categories
   are covered by active prompt-variation tests and deterministic handlers.
3. **Multilingual benchmarks pre-translate the same semantic prompt** instead
   of literal English-first translation. Belebele, XNLI, XCOPA, MGSM, FLORES,
   TyDi-QA, and XQuAD all use this pattern. We mirror it: each new prompt
   matrix contains natural per-language phrasings (not literal translations),
   so transliteration and idiom handling exercise the real solver paths.
4. **Rust test scaffolding has good options.** `rstest` and `test-case` are
   the most ergonomic table-driven libraries, `insta` is the cheapest path to
   regression-locking deterministic symbolic outputs, and `datatest-stable`
   can drive prompts from external YAML/JSON files when the matrix grows. We
   intentionally kept zero new test dependencies: formal-ai's existing
   `calculator_delegation.rs` pattern (`for (prompt, expected) in [..]`) is
   already enough to express 5–10 × 4 matrices with no extra crates, and we
   reused exactly that pattern. Adding `rstest`/`insta` is a clean upgrade
   path captured in `ARCHITECTURE.md` for a follow-up issue.
5. **Architecture references.** The issue's "doublets-rs/doublets-web data
   store" line maps to
   [`link-foundation/doublets-rs`](https://github.com/link-foundation/doublets-rs)
   and [`link-foundation/doublets-web`](https://github.com/link-foundation/doublets-web).
   The "Wikidata P/Q-ID formalization" line maps to public Wikidata
   `P-id` (property) and `Q-id` (item) IRIs at
   `https://www.wikidata.org/wiki/Property:PXX` and
   `https://www.wikidata.org/wiki/QXX`. `ARCHITECTURE.md` references these as
   the long-term external sources of formalization anchors.

## Root Cause

Before this change, the test catalog had grown organically:

1. each test pinned **one** specific prompt to **one** intent;
2. most categories had English coverage but only a couple of Russian
   variations and almost no Hindi or Chinese surface tests;
3. the architecture story lived split across `VISION.md`, `GOALS.md`, and the
   per-issue case studies, so a contributor had to read five documents to
   understand the design;
4. there was no single document describing the full pipeline
   (context → Links Notation → Wikidata formalization → temperature →
   clarification → growable memory → trigger-style computation over links →
   formalization-driven translation between natural and programming
   languages).

That worked for shipping one issue at a time, but the issue reporter wanted
both a horizontal expansion (more prompts per language) and a vertical anchor
(`ARCHITECTURE.md`) so the project can keep growing without losing coherence.

## Design Decisions

- **Reuse the table-driven pattern.** `tests/unit/specification/calculator_delegation.rs`
  already demonstrated the idiomatic Rust pattern for 5–6 variations per
  language. We extend the same pattern instead of adding a new test
  framework. Zero new dependencies.
- **Pin language detection per variant.** Every multilingual matrix asserts
  the right `language:*` evidence link is emitted, so a regression that
  collapses Hindi prompts into the English fallback is caught immediately.
- **Active assertions for competitor-derived prompts.**
  Variations from competitor categories become active `#[test]` assertions in
  `prompt_variations.rs`. The solver now has deterministic handlers for
  summarization, brainstorming, factual Q&A, multi-turn coreference, and
  roleplay so these prompts exercise real behavior instead of documenting a
  gap.
- **`ARCHITECTURE.md` is the single source of truth for design.**
  `VISION.md` retains the product/values angle, `GOALS.md` and `NON-GOALS.md`
  retain scope, `REQUIREMENTS.md` retains the issue-by-issue traceability
  matrix, and the new `ARCHITECTURE.md` documents the actual pipeline so
  reviewers do not have to triangulate.
- **Architecture additions are explicit but non-breaking.** The new
  vision section names Wikidata P/Q-IDs, the temperature knob, doublets-rs
  persistence, and natural-language-skill compilation as the *direction*; no
  surface contract changes in this PR.
- **The competitor research is a permanent artifact.** Future PRs that add a
  category should append to
  [`competitor-test-research.md`](raw-data/competitor-test-research.md)
  rather than starting fresh, so the rationale is preserved.

## Verification Plan

- Existing focused regression suites stay green:
  - `cargo test --test unit chat_surface -- --nocapture`
  - `cargo test --test unit code_generation -- --nocapture`
  - `cargo test --test unit multilingual -- --nocapture`
  - `cargo test --test unit calculator_delegation -- --nocapture`
- New matrix runs:
  - `cargo test --test unit prompt_variations -- --nocapture`
- Full local CI before PR finalization:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-features --verbose`
  - `cargo test --doc --verbose`
  - `rust-script scripts/check-file-size.rs`
- Documentation regression: existing `tests/unit/docs_requirements.rs`
  enforces that `VISION.md`, `GOALS.md`, `NON-GOALS.md`, and `REQUIREMENTS.md`
  remain present and well-formed; `ARCHITECTURE.md` is added under the same
  pattern.
