Parent: #651. Full planning context with audits: `docs/case-studies/issue-651/` (this issue is E55 in `proposed-issues.md` there; added by PR #652).


**Problem**

`ARCHITECTURE.md` §16 has carried one open question since the E20 batch:
"arbitrary natural-language programming beyond the supported subset" of
`src/skill_compiler.rs`. `docs/USER-JOURNEYS.md` F2 describes the journey — a
user states a multi-step procedure in plain language ("when I paste a URL,
fetch its title, translate it to Russian, and store both") and the system
compiles it into a typed, executable skill. Today the compiler handles typed
trigger/response and a bounded multi-step shape; procedures outside that
shape fall back to formalization without compilation. This is the "five rule
shapes ending in compiled natural-language skills" pillar taken to its
stated conclusion.

**Approach**

1. Reuse the solver's own decomposition (the same move as the general
   planner in E35/#654): formalize the stated procedure into ordered
   sub-requirements, then map each sub-requirement onto the skill
   compiler's existing typed step vocabulary (fetch, transform, translate,
   store, reply, condition).
2. Where a step has no existing vocabulary entry, fail honestly with a
   named gap ("no compiled capability for X") and record a
   `skill_gap` event — never silently drop a step.
3. Grow the step vocabulary as seed data (`data/seed/`), not Rust match
   arms, so new step kinds are data edits (the operation-vocabulary
   precedent from E33).
4. Multilingual from the start: the same procedure stated in en/ru/hi/zh
   compiles to the same skill links (round-trip guard).
5. Compiled skills remain inspectable: "why did you do that?" cites the
   compiled steps and their source sentence spans.

**Existing components**

- `src/skill_compiler.rs` typed/multi-step compiler with native lowering.
- `src/intent_formalization.rs` + solver decomposition (steps 2–5).
- `data/seed/operation-vocabulary.lino` — the multilingual step-vocabulary
  pattern.

**Acceptance criteria**

- `cargo test arbitrary_skill_compilation` — a ≥ 4-step procedure phrased
  freely (not matching any existing compiler template) compiles, executes
  end to end, and re-states its steps on request; the same procedure in
  Russian compiles to the same skill links.
- A procedure containing one uncompilable step yields the honest named-gap
  reply plus a `skill_gap` event, and compiles nothing partially.
- `ARCHITECTURE.md` §16 open question removed; `docs/USER-JOURNEYS.md` F2
  marked with its new status.

