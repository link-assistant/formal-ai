**Problem.** Once Formal AI can code, that skill must speed up Formal
AI's own development (issue #914) — the reason coding comes first. The
self-hosting metric (#657) measures the share honestly from its
baseline, and the promotion protocol (#656) gates changes, but no
recurring loop routes real repository work through Formal AI itself.

**Approach.** After E69's write-effect rungs pass, establish the loop:
each release cycle, at least one real, reviewable repository change
(documentation sync, seed data update, test addition, or a small fix) is
produced by Formal AI — directed through the hive-mind path from E74 or
the Agent CLI directly — landing as a normal reviewed pull request. The
self-hosting ledger in `data/meta/` records each contribution, the share
is reported per release, and the target only ratchets up. Every change
passes the same review, CI, and promotion gates as human work.

**Existing components.** The #657 self-hosting ledger and metric;
`src/promotion.rs` (#656); `src/self_source_links.rs` and
`src/self_ast_census.rs` (#673); the agentic-coding recipe
(`data/meta/agentic-coding-recipe.lino`, #468); E69's ladder and E74's
integration gate.

**Acceptance criteria.**
- At least one merged pull request per release cycle is authored by
  Formal AI end to end, with replayable session evidence.
- The self-hosting share is reported per release from the ledger and is
  wired as a ratchet (may not silently decrease).
- Every self-authored change passes unmodified review, CI, and promotion
  gates.
- Depends on E69 and E74.

---

Part of the issue #914 vision-planning batch (E69-E77). Parent: #914. Case study, gap analysis, and design rules: `docs/case-studies/issue-914/` on the issue #914 branch (pull request #915).

