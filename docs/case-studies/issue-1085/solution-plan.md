# Issue #1085 solution plan

One pull request on one branch, landed in pushes; CI is the test bed (the
maintainer's machine cannot build the crate). Each push is a coherent slice and
this file says which slice carried what.

## Push 1 (this document's revision)

1. **#1081 remainder.** Re-read the four failing `main` jobs: the Auto Release
   failure was the self-development gate itself; two macOS jobs died on the
   1400 s execution budget; the newest failure was the credential probe that
   #1082 added, which reads crates.io's cookie-only `/me` 403 as a revoked
   token. Fix: the probe records `unknown` with the reason and never sends the
   token; the archive budget matches the measured spread.
2. **D3 metric version 3.** `self-hosting-attribution.rs` (model trailer,
   hosted-model markers, behaviour-only paths), `self-hosting-replay.rs`
   (`--replay-epoch`, pull-request author), ledger header, fixtures.
3. **D3.5.** Gate removed from `release.yml` and `version-and-commit.rs`;
   `self-development-status.yml` runs the floor and the kernel shrink rule.
4. **D1.4.** `kernel-ratchet.lino` and `check-kernel-ratchet.rs`, registered as
   a rust-stage gate; raisable handler-ledger ceilings retired.
5. **D4 (first half).** `cargo check --lib` per `.rs` leaf, shared target dir.
6. **D5.4, documents.** Upstream numbers beside 13/13; VISION, GOALS,
   NON-GOALS, ROADMAP, REQUIREMENTS, traceability, README, troubleshooting.

## Push 2

- D2.1 `link_edit_rules.rs` and its rule file; `structured_edit` delegates.
- D3.4 `--replay-epoch` over 84 releases: every one 0.00 % (204 commits without
  `Formal-AI-Model`, 164 hosted-model sessions); version 1 and 2 rows archived
  verbatim beside the ledger to stay under the 1,500-line cap.
- D5.3 diagnosis from the scheduled run's log (`ci-evidence/`): HumanEval/0's
  candidate lost `from typing import List`; MBPP/2's "signature" was an
  `assert` call. Both fixed in `program_synthesis.rs` with an upstream-shaped
  regression test.
- Push-1 CI feedback folded in.

## Following pushes on the same branch

7. **D3.4 rows.** Run `--replay-epoch` in CI, read the restated rows from the
   status workflow's output, commit them.
8. **D2.1 link-substitution edits.** A `code_edit_rules` module: parse the
   target through the meta-language CST (`crate::coding::cst`), apply one of
   three `.lino` rule shapes (member insertion, literal replacement, identifier
   rename) over the links network, `reconstruct_text`, `rustfmt`, `cargo
   check`, `cargo test`; `structured_edit.rs` delegates to it.
9. **D4 (second half).** `cargo test --test unit <module>` per leaf; composite
   nodes apply both children's diffs to one tree and compile; depth-3
   requirement-shaped prompts resolved through the self-AST census.
10. **D1.1-D1.3.** Seed into `LinkCliLinkStore` at startup behind a feature
    flag first, then default; link-query routing for cue and precedence
    lookups; interpreter for rule shapes 1 and 5; migrate the smallest pending
    handlers and lower the ceilings as they go.
11. **D5.1-D5.3.** Frontier inputs from `external-results.lino`; red on a
    falling suite; the HumanEval task-0 transfer failure explained and fixed.
12. **D6-D9** as sub-issues blocked by this one.

## Existing components surveyed

- `src/link_store.rs` (`LinkCliLinkStore`, doublets-rs via link-cli) -- the
  store the runtime must read from, today written after each `.lino` write.
- `src/recipe_interpreter.rs`, `src/substitution.rs`,
  `src/links_substitution_query/` -- the pieces a generic rule interpreter
  composes.
- `src/coding/cst.rs`, `src/agentic_coding/self_ast.rs` -- meta-language CST
  round trip for Rust and JavaScript (`reconstruct_text() == source`).
- `src/agentic_coding/structured_edit.rs` -- byte-offset member insertion to be
  replaced by link rules.
- `src/substitution_compiler/` -- rule IR to Rust, JavaScript and WASM.
- `scripts/self-hosting-metric.rs` family -- attribution and ledger.
- `experiments/issue_1028_agent_cli_ladder/` -- the change-shaped ladder.
