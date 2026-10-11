**Problem.** The vision requires a minimum core of algorithms plus a data
seed with metadata rich enough to problem-solve the way people do (issue
#914). The meta algorithm is already executed as data, but the #559
mandate is unmet: about 19,600 lines across 40 files remain in
`src/solver_handlers/` after the #699 migration, and no audit defines
which metadata each seed record must carry.

**Approach.** Define the core boundary explicitly (meta algorithm, link
store, interpreters, surfaces) and put every handler on a burn-down
ledger: migrate to seed rules, promote into the documented core with a
stated reason, or delete. Gate the boundary with a ratchet script in the
style of `scripts/check-hardcoded-language.rs`. In parallel, audit seed
records against a declared metadata schema for problem solving — roles,
preconditions, effects, units, examples — taking FrameNet's
frame-and-role shape and Wikidata's typed properties as vocabulary
sources, and record per-record gaps as data.

**Existing components.** `src/recipe_interpreter.rs` executing
`data/meta/recursive-core-recipe.lino`; the #699 migration machinery and
its handler registry; `data/seed/` (117 lino files); the burn-down-gate
script pattern; FrameNet and Wikidata vocabularies.

**Acceptance criteria.**
- A documented core boundary exists and a ratchet script enforces that
  handler code outside it only shrinks.
- Every remaining handler has a ledger entry: migrated, promoted with
  reason, or deleted.
- The seed metadata schema is documented and at least the concept records
  used by the coding path satisfy it, with gaps recorded as data.
- The regression floor holds.

---

Part of the issue #914 vision-planning batch (E69-E77). Parent: #914. Case study, gap analysis, and design rules: `docs/case-studies/issue-914/` on the issue #914 branch (pull request #915).

