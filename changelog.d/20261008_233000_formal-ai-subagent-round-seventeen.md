### Changed

- PR #1188 round 17:
  - **Formal AI never takes an action it was not asked for (R1188-U17),** in both roots:
    - a replace list reads its target from any clause (G89), and compares quoted segments after unescaping prose newlines (G88);
    - an edit that names two or more target files outside quotes is declined with the seeded `request_several_edit_targets` answer (G91);
    - the reply language is detected from the text outside quotes (G92);
    - a cue word quoted whole is data, not a request word;
    - a routed Write that carries an unquoted edit cue is not a whole-file write;
    - an odd `"` after a letter counts as unpaired.

    A property test of 240 generated requests per root (seed 1188) pins it.
  - **Requirement listing chat route (R1188-U20):** "List the requirements of this issue: <text>" lists what the extractor finds, in five languages. It is a `requirement_listing` rule set with two generic rule-language additions: the `command_head` subject and the `value <name> transform <transform>` source. A pasted phrase never routes.
  - **Text capabilities (R1188-U18, U19, U21):**
    - formalization of whole web pages;
    - round-trip translation over all 20 ordered language pairs;
    - dependency-based summarization.

    Each has a measured ratchet that only rises.
  - **Planner twins (R1188-U16):** `scripts/check-planner-twins.mjs` (gate `check_planner_twins`) counts the functions `js/agentic/` exports with no Rust twin, by snake_case name, a `Mirrors` citation, or a declared Rust built-in or dependency. All 88 that had none now have one, so the ceiling in `data/meta/planner-twin-ratchet.lino` is 0:
    - 14 gained a citation;
    - 11 were extracted from inline Rust into named functions, which also removed duplicated code;
    - 27 were declared as standing for a Rust built-in or a dependency crate;
    - one Rust twin was written: `module_from_document`;
    - one unused export was deleted.

    Formal AI made the 55 doc-comment and ledger edits. It failed twice, and both were fixed in both roots: G96, an anchor holding a literal `\n`; and G97, a Spanish marker matched inside a file name.
  - **Long tests first and parallel at test level in the browser suite (R1188-U10, U11):**
    - the local web E2E legs are planned longest-first by `scripts/plan-test-shards.mjs` from `data/meta/playwright-test-durations.lino`, recorded from CI by `ci-durations.mjs --playwright-files`;
    - the config runs its long specs first in a leading `chromium-long` project;
    - the speed gate now rejects Playwright's count-based `--shard=i/n`.
    - the suite runs `fullyParallel` (R1188-U11). The two specs whose tests share state made in `beforeAll` keep their tests in one worker.
  - **Formal AI tally (R1188-U13):** `scripts/tally-formal-ai-dogfood.mjs` generates `formal-ai-tally.md` from the dogfood ladder: tasks, outcomes and resolutions, per agent. Gate `check_formal_ai_tally` fails on an unresolved failure, and on a fixed one whose regression test is neither named in its row nor cites its id.
  - **Generated files are checked in CI (R1188-U8):** `scripts/check-generated-files.mjs` (gate `check_generated_files`) fails on a file whose header names a generator that no gate or workflow runs, so a hand edit of a generated file cannot be merged. New gates run three checks that no gate ran before: `check_llm_task_parity`, `check_fact_captures` and `check_requirement_coverage`.
  - **Generalize, don't specialize (R1188-U1):** `scripts/check-prompt-specialization.mjs` (gate `check_prompt_specialization`) counts the prompts that tests send to Formal AI held verbatim in code. These are branches or canned answers keyed to one prompt. The ratchet holds the count at 23 and it only falls.
  - **Requirement audit:**
    - the coverage map now includes the owner's 2026-10-08 messages (R1188-U18 to U26), 115 requirements in all;
    - R1188-U15, U22, U24, U25 and U26 are implemented: the working rules are pinned to their record and to the tools that keep them by `rust/tests/web/pull-request-1188-working-rules.test.mjs`;
    - the closure audit grounds the `machine` response audience.
