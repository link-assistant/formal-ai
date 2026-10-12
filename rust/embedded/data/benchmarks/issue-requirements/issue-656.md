Parent: #651. Full planning context with audits: `docs/case-studies/issue-651/` (this issue is E37 in `proposed-issues.md` there; added by PR #652).


**Problem**

Every self-improvement loop in the codebase is proposal-only by design:
`src/self_improvement.rs` proposes seed rules but never writes `data/seed/`,
`src/meta_self_improvement.rs` defaults to `Off`, `src/self_healing.rs`
produces human-gated `RepairCase`s, and dreaming amendments live only in
memory events. The vision's self-coding rung 3 needs an explicit,
deterministic **promotion protocol**: a proposal that passes its benchmark
ratchets and CI may be promoted into seed data automatically, while draft PRs
and human review remain the outer gate. Without it, learning cannot compound.

This issue is also the missing tracker for R385: REQUIREMENTS.md still says
arbitrary auto-learning is "tracked by issue #558", but #558 was closed by
the deliberately human-gated PR #637, so the capability currently has no
open tracker at all (incomplete-work audit, item 8).

**Approach**

1. Define a `promotion` event protocol in the meta language: proposal link →
   benchmark evidence links (which ratchets ran, at what floor) → promotion
   decision → applied change, all appended to the event log.
2. Implement `formal-ai improve --promote` (dry-run by default, `--apply`
   with confirmation like `memory dream --apply --confirm`): collects open
   proposals, replays their gates (coding-modification suite, industry
   suite, unit specs), and materializes accepted ones as `.lino` seed edits
   on a branch — never a direct push.
3. Rejected proposals persist with the failing evidence (the R425
   `dreaming_candidate_failure` pattern).
4. Wire the branch/PR step through the same Agent-CLI path E36 exercises, so
   a promotion lands as an ordinary reviewed pull request with a changelog
   fragment.

**Existing components**

- `src/self_improvement.rs`, `src/meta_self_improvement.rs`,
  `src/self_healing.rs`, `src/dreaming.rs::MetaAlgorithmAmendment`.
- Ratchets: `data/benchmarks/*.lino` `minimum_pass_count` floors.
- Destructive-action gates: `require_destructive_confirmation`,
  `write_full_memory_backup`.

**Acceptance criteria**

- `cargo test promotion_protocol` — a synthetic proposal that passes its
  gates is materialized as a seed edit in a temp workspace; one that fails a
  ratchet is preserved as a failure record and **not** applied.
- Promotion events round-trip through the bundle export/import.
- `formal-ai improve --promote` (dry run) prints the plan without touching
  files; `--apply` without `--confirm` refuses.
- Documentation: `docs/meta-algorithm.md` gains a promotion section pinned by
  a traceability test.



