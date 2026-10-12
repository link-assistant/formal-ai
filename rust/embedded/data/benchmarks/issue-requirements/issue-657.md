Parent: #651. Full planning context with audits: `docs/case-studies/issue-651/` (this issue is E38 in `proposed-issues.md` there; added by PR #652).


**Problem**

The vision's self-coding rung 4 says each release should report what share of
its changes was authored by Formal AI itself, and that share should ratchet
upward (the discipline the benchmark suites already apply to solving
ability). No such measurement exists. Prior art (Aider's per-release
self-written percentage; SICA's benchmark-gated self-editing) shows the
metric is both computable and motivating.

**Approach**

1. Define authorship attribution: commits whose recorded session evidence
   (E36 self-coding runs, E37 promotions) links them to a Formal AI session
   count as self-authored; measure in changed lines per release window.
2. Implement `scripts/self-hosting-metric.rs` (rust-script, like the other
   release scripts) that reads the git history between release tags plus the
   committed session ledgers and emits the percentage.
3. Publish the number in release notes via the existing
   `create-github-release.rs` step, and record it as a `.lino` ledger row so
   the trend itself is links data.
4. Start honest: 0% is an acceptable first value; the ratchet is monotonic
   non-decreasing over a trailing window, not a hard floor.

**Existing components**

- `scripts/{get-bump-type,version-and-commit,create-github-release}.rs` —
  the release pipeline to extend.
- `docs/case-studies/issue-538/agent-cli-session*.json` — the session-ledger
  precedent for attribution evidence.

**Acceptance criteria**

- `rust-script scripts/self-hosting-metric.rs --since <tag>` prints a
  deterministic percentage from committed data (covered by a unit test with
  a fixture repo/ledger).
- The release workflow emits the metric into the GitHub release body.
- A `data/meta/self-hosting-ledger.lino` row is appended per release and
  pinned by a specification test.



