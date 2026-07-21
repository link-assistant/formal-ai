# Issue #701 — closing the auto-learning adoption gap

- Session: `issue-701-claude-20260721`
- Agent: formal-ai (Claude Opus 4.8) via `/solve`
- Issue: <https://github.com/link-assistant/formal-ai/issues/701>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/817>
- Case study (measurements, tables, research): `docs/case-studies/issue-701/`

Every claim below is either backed by a committed artifact or by a command that
can be re-run from the repository root. Where something is a narrowing rather
than a result, it says so.

## 1. What was actually broken

Two symptoms, both measured before anything was changed.

**The Trends frontier never moved.** `examples/issue_701_frontier_census.rs`
replays the committed Google Trends corpus (10 topics × 4 languages × 2
variations) through the production solver and counts intents:

```text
total=80 unknown=60 answered=20
```

The same 60 prompts were recorded as unroutable run after run, and nothing was
ever derived from them.

**Dreaming amendments were decoration.** `examples/issue_701_amendment_body.rs`
solves a prompt plainly, solves it again with a matching retained amendment,
strips the appended compliance line, and diffs the bodies:

```text
--- latex: solve a new recurrence proof
plain intent   = unknown
amended intent = unknown
body differs beyond the appended line = false
```

Root cause: `solve_with_amendment_records` prepends a
`Standing requirement (...)` user turn; `solve_with_history` records it as a
`prior_turn:user` event; **no downstream handler ever reads that event**. The
rule reached the log and the prose, never the routing.

## 2. What changed

| Requirement | Registry row | Change | Proof |
| --- | --- | --- | --- |
| Adoption contract | R484 | `formal-ai learn cycle --frontier google-trends --dry-run [--proposals]` (`src/learning_cycle.rs`, `src/cli_learn.rs`) | `tests/unit/issue_701_learning_adoption.rs` |
| Capability delta | R485 | `data/meta/learning-adoption-ledger.lino`, tool-authored and byte-pinned | ledger render test |
| Trends gap closed | R486 | learned request-opener surfaces in `data/seed/` | `the_trends_corpus_unknown_rate_is_ratcheted_to_zero` |
| Decoration path deleted | R487 | `src/dreaming_application.rs`: one selection rule, one projection, intent reclassification | `tests/unit/issue_701_dreaming_amendment_class.rs` |
| Periodic proposal-only run | R488 | idle dreaming record + `.github/workflows/learning-cycle.yml` | `every_idle_dreaming_run_leaves_a_proposal_only_learning_cycle_record` |
| Durable failure records | R489 | frozen frontier record + blocked classes | frontier-record tests |

## 3. Verification run in this session

| Command | Result |
| --- | --- |
| `cargo test --release` | 1966 unit tests passed, 0 failed, 2 ignored |
| `rust-script scripts/check-hardcoded-language.rs` | in sync (1327 entries) |
| `rust-script scripts/check-file-size.rs` | all files within limits |
| `rust-script scripts/check-changelog-fragment.rs` | passed (1 fragment) |
| `rust-script scripts/check-associative-terminology.rs` | passed |
| `cargo fmt --all -- --check` | clean after `cargo fmt --all` |
| `rust-script scripts/check-worker-line-budget.rs` | 26 819 of 26 819 lines (re-baselined for the mandated mirror) |
| `node experiments/issue_701_worker_mirror_check.mjs` | `checked=90 mismatched=0` |

The last row is the evidence for the worker-mirror compaction: the JS
`extractTermInformationRequest` was collapsed from three candidate loops into a
single candidate list to hold the line ratchet, so the script evaluates the whole
worker bundle in a VM context and compares the compacted function against the
loop version it replaced over all 80 trends prompts plus 10 degenerate inputs
(bare affixes, empty text, affix-only prompts). No input separates them.

## 4. Narrowings, stated honestly

- The cycle derives one *class* of knowledge — request-opener surfaces —
  because that is the class the recorded frontier actually consists of. The
  contract is class-agnostic; the tested coverage is not.
- Requirement 4's delta is a change in routing and evidence, not a rewrite of
  the answer body by the standing rule. A deterministic engine cannot invent a
  body from a rule it has no handler for; claiming it did would reproduce the
  decoration problem in a new costume.
