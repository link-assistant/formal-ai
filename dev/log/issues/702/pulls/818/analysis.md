# Issue #702 — dialogue world model (current state, target state, difference)

- Session: `issue-702-claude-20260721`
- Agent: formal-ai (Claude Opus 4.8) via `/solve`
- Issue: <https://github.com/link-assistant/formal-ai/issues/702>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/818>
- Evidence bundle: this file plus
  [`docs/case-studies/issue-702/`](../../../../../docs/case-studies/issue-702/)
  (raw issue/PR JSON, requirements matrix, per-requirement solution plans)

Every claim below is either backed by a file committed in this pull request or
by a command that can be re-run from the repository root. Where something is a
design decision rather than an observation, it says so.

## 1. What the issue asked for

Issue #702 (E60, child of #651) implements the per-requirement plans drafted in
issue #649. PR #675 had already landed the *substrate* — `src/world_model.rs`:
contexts as links networks, statements with relative-meta-logic truth values,
dependency recalculation, STRIPS-style actions. Nothing fed that substrate from
a conversation. The eight requirements are tabulated in
[`docs/case-studies/issue-702/requirements.md`](../../../../../docs/case-studies/issue-702/requirements.md)
and mapped to `R702-1 … R702-10` in [`REQUIREMENTS.md`](../../../../../REQUIREMENTS.md).

## 2. Root cause of the gap

The world model was reachable only from Rust tests. No conversation turn wrote a
statement into a context, no vocabulary separated a wish ("I want the door to be
open") from a fact ("the door is closed"), and no chat surface could answer a
question about the difference between them. Two pieces were missing: the
**dialogue → links ingestion path** and a **handler that answers from the
difference**.

## 3. What this pull request adds

| Path | What it is |
| --- | --- |
| `src/world_model_atoms.rs` | Utterance classification (fact / wish / confirmation / correction / state query) and `text → SubstitutionLink` extraction |
| `data/meta/cue-lexicon.lino` | The recognition vocabulary as reviewable link data (`world_state_*` cue sets, en/ru/hi/zh) — no Rust phrase tables |
| `src/world_model_dialog.rs` | `DialogueWorldModel`: current/target contexts with per-turn provenance, hash-chained append-only `SyncEvent` log, `difference`/`remaining`, `depends_on`/`revise_statement`, `forecast`, `merge_from`/`split_current`, `record_world_model` |
| `src/solver_handlers/world_state.rs` | The `world_state` contextual handler, answering from the current→target difference with `world_state:*` evidence links |
| `data/benchmarks/world-state-tracking-suite.lino` | 16 self-authored bAbI-style state-tracking dialogues with a `minimum_pass_count` ratchet |
| `examples/world_state_dialogue.rs` | Runnable multi-turn reproduction (the `chat` subcommand is single-shot) |

The feature is **trace-only until opted in**: `SolverConfig::world_model_mode`
defaults to `WorldModelMode::Off`, and with it off no `world_state:` link is ever
emitted (`the_world_model_handler_is_inert_until_the_knob_is_opted_in`).

## 4. Verification actually run

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | clean (after the private intra-doc link fix) |
| `cargo test --test unit` | 1975 passed, 0 failed, 2 ignored |
| `cargo run --example world_state_dialogue` | prints the off/track comparison in all four languages |

Two committed artifacts had to be regenerated because this branch adds modules
and seed responses:

```bash
cargo run --example regenerate_self_ast_census   # 292 documents
python3 scripts/close-total.py                   # unresolved_distinct: 0
```

## 5. Attribution

The commits in the range `f32db790..HEAD` were authored in this session and
carry `Formal-AI-Session: issue-702-claude-20260721` together with
`Formal-AI-Evidence` pointing at this file, per
[`CONTRIBUTING.md`](../../../../../CONTRIBUTING.md) § *Recording self-authorship*.
The branch's first commit (`f32db790`, "Initial commit with task details") was
written by the task harness, not by this session, and is deliberately left
unattributed.

Reproduce the measurement with:

```bash
python3 experiments/self_hosting_ratchet_replay/replay.py v0.301.0 HEAD
```
