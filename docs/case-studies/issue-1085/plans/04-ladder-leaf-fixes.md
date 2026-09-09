# Plan 04 -- the 15 leaves that fail for two reasons (#1095, #1096; stretch #1101)

The ladder record is 15 of 32 leaves. Of the 17 failures, 8 are #1095 and 7
are #1096 (`docs/case-studies/issue-1085/ci-evidence/ladder-leaf-failures.md`).

## #1095 -- a continuation cue resumes the task

**Actual gap** (read from `src/agentic_coding/planner.rs:158-200, 621-650`):
`compacted_agent_task` recovers the objective only when the assistant turn
immediately before the cue carries a `Conversation summary:` envelope -- the
compaction case. In the ladder the Agent CLI sends `Continue if you have next
steps` after a **tool result**; there is no envelope, the function returns
`None`, `effective` is the cue itself, and it falls through to
`plan_web_search_step`. The English string is also matched exactly, so ru/hi/zh
cues never enter this path at all.

- [ ] 1. Seed role `agentic_continuation_cue` in the lexicon, four languages
      (en "continue if you have next steps", "continue", "go on"; ru
      "продолжай", "продолжи, если есть следующие шаги"; hi "जारी रखें"; zh
      "继续" / "如果还有下一步请继续"). Exact-phrase entries, not stems, so an
      ordinary request containing the word keeps its meaning (the existing
      comment's constraint).
- [ ] 2. `continued_agent_task(messages, latest)`: if `latest` is only a cue
      (whole normalized text matches the role), return the objective of the
      most recent earlier user turn that is *not* a cue -- after the compaction
      envelope path has been tried first (keep that; it is more specific).
- [ ] 3. Replace the exact string at `planner.rs:624` with the role check.
- [ ] 4. Tests (`tests/unit/issue_1095_continuation_cue.rs`): task turn, tool
      result, cue in each of the four languages -> the plan resumes the task
      (assert `agentic_task` trace equals the task, and no `web_search`
      capability in the plan); an ordinary request that merely contains
      "continue" still routes normally; the compaction-envelope case still
      works.

## #1096 -- an edit of an existing file is not a generation task

**Gap** (`src/agentic_coding/code_task.rs:79-90`): `rust_source_for_task`
claims any task that mentions a program-request role ("keep it valid Rust")
and names a `.rs` path. A replacement in an existing tracked file is then
verified against the artifact the generator synthesised.

- [ ] 1. In `plan_generated_source_step`, before claiming: if the task carries
      an edit cue (role `ROLE_EDIT_ACTION` / the `compose_edit_request` path
      already recognises "replace X with Y") **and** the named path exists in
      the workspace, do not claim; let `compose_edit_request` /
      `structured_edit` own it.
- [ ] 2. Tests: the leaf's exact task shape (`In the file src/x.rs, replace
      "A" with "B". Change only that file and keep it valid Rust.`) plans an
      edit step, not a generated-source step, in four languages; a genuine
      "write a new file src/y.rs with a function that returns 3, keep it valid
      Rust" still plans generation.

## Ratchet

- [ ] After both land, run the ladder (`workflow_dispatch`, depth 5). Raise
      `leaf_nodes_passing` in `data/meta/ladder-ratchet.lino` to the measured
      value in its own reviewed commit, with the run URL in the message.
      Expected: 15 -> about 30 (the two remaining failures are mechanism C,
      "reported without being made", which plan 03's item state addresses).

## Stretch: #1101

`data/seed/handler-rules.lino` docs rule matches all four languages but
`handler-precedence.lino` lets `web_search` win when the prompt is not English
because the docs rule's `role_prefix` list has only the English lead. Add the
ru/hi/zh leads; test the four prompts from the issue table.
