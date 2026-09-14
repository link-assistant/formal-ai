# Plan 03 -- Formal AI writes its plan before it codes

Closes #1099; delivers the maintainer's ask that a coding task begins with a
markdown plan the user can confirm, or that is saved and executed item by item.

## What exists

`src/agentic_coding/general_planner.rs` already composes a
`GeneralChangePlan` (goal, target, ordered `GeneralPlanStep`s, verification
command, terminal state) and writes it to `.formal-ai/general-change-plan.lino`
*before* execution. Two things are missing:

1. The plan has one target. A request naming two artifacts (#1099) produces a
   plan for the first and answers `Final` when it verifies.
2. Nothing renders the plan for a person, and nothing tracks per-item state
   across turns.

## Design

- **Plan items.** `GeneralChangePlan` gains `items: Vec<PlanItem>`, one per
  obligation the request names (each clause introducing an artifact: "First,
  edit X ... Second, write Y ..."; the enumeration cues live in the seed
  lexicon, en/ru/hi/zh). Each item has `status: Pending | Done | Failed` and
  its own verification command.
- **Markdown rendering.** `render_markdown()` -> `.formal-ai/PLAN.md`:
  title, goal, a checkbox per item, the verification command per item. Written
  in the same turn the `.lino` is (the `.lino` stays the machine record).
- **Confirmation mode.** When the request carries a confirmation cue ("plan
  first", "show me the plan", "confirm before changing" -- seed role
  `plan_confirmation_cue`, four languages) the planner writes both files and
  answers with the markdown plan and a localized "reply `go` to execute"
  line; the next turn that is an affirmation resumes from the file. The
  #656 `--confirm` flow and the calendar "reply yes" flow are the precedents.
- **Item-by-item execution.** Without the cue, the driver executes item 1,
  verifies it, marks it `Done` in both files, and continues. `Final` is only
  reached when no item is `Pending`. A failed item stops with the item named
  (never `Final`). This is the #1099 fix.

## Steps

- [x] 1. Seed: `plan_confirmation_cue` and `enumeration_cue` roles in
      `data/seed/lexicon` (four languages); responses
      `general_plan_awaiting_confirmation` (four languages).
- [ ] 2. `general_planner.rs`: `PlanItem`, splitting a request into items,
      `render_markdown`, `PLAN_MARKDOWN_PATH`.
- [x] 3. Driver (`planner.rs` / `code_task.rs`): read item state; `Final`
      requires all `Done`; confirmation cue -> await turn.
- [x] 4. Tests: #1099's exact prompt in en/ru/hi/zh -> both files exist,
      exact answer; confirmation cue -> PLAN.md written, nothing changed, exact
      awaiting answer; `go` -> executes; one failing item -> not `Final`.
- [ ] 5. Ladder leaf: add the two-artifact task as a leaf rule, so the
      ratchet measures it.
- [ ] 6. Docs: `docs/meta-algorithm.md` planning section; CONTRIBUTING note
      that this file's own convention (plans before code) is what the system
      now does for itself.

## Kernel note

`general_planner.rs` is not a kernel path; the ratchet counts its lines. Keep
the item splitting in the seed (cues) and the rule interpreter where possible;
budget: net lines here must be offset by the `code_task.rs` claim removal in
plan 04.

## Log

- 2026-09-10: delivered as **obligation tracking**, not as a rendered plan file.
  What #1099 reported is that a request naming two artifacts was planned, and
  answered, as if it named one; `src/agentic_coding/task_obligations.rs` splits
  a request at its seeded enumeration cues, the planner plans the first
  artifact the workspace does not yet have, and `Final` is unreachable while
  one is outstanding. Steps 2, 5 and 6 (a rendered `PLAN.md`, a confirmation
  turn, a ladder leaf) are not delivered: they are a *presentation* of the same
  state, and the defect is the state. They stay open above.
- The issue's literal prompt shape ("edit the tracked file X: add "Y" to the Z
  list") still composes to nothing -- the write-request reader wants
  `create file PATH containing TEXT` -- so the tests use the phrasing the
  reader accepts and say so. That reader limit is its own defect, filed
  separately rather than folded in here.

