**Problem.** The system must learn to discover enough knowledge from the
internet and other sources to solve all tasks, coding first (issue
#914). Retrieval today is answer-oriented: search results are fused and
presented, but retrieved material does not become reusable, verified
coding capability, and open issues #873 and #896 track exactly this
demand.

**Approach.** Add the loop: when a coding task hits a recorded skill gap,
plan a research query, fetch through the provenance-tracked source cache,
formalize the retrieved material into the meta language, compile a
candidate procedure (#897 machinery), verify it by execution in the
bounded workspace, and keep it only when execution proves it — with
source, license, and fetch metadata attached. Failed research rounds
update the gap record so "not knowing is not the end" (#873): the gap
itself schedules the next round.

**Existing components.** `src/web_search_core.rs` and
`src/search_fusion.rs`; `src/source_fetch.rs` provenance cache;
`src/knowledge.rs` oracles (Rosetta Code, Wikifunctions, Stack Overflow
snapshots); `src/skill_procedure.rs` and verified procedures (#897);
`src/program_skill_gap.rs`; open issues #873 and #896.

**Acceptance criteria.**
- At least one coding task that fails as a skill gap is solved end to end
  by the research loop, with the learned procedure kept with full
  provenance and replayed deterministically from cache in CI.
- Procedures learned from research are marked as such and pass the same
  execution verification as hand-seeded ones.
- Live fetching stays opt-in; offline mode replays the cache.
- Depends on E69 for the execution-verification path.

---

Part of the issue #914 vision-planning batch (E69-E77). Parent: #914. Case study, gap analysis, and design rules: `docs/case-studies/issue-914/` on the issue #914 branch (pull request #915).

