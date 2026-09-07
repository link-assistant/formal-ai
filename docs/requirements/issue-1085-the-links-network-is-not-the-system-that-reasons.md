## Issue #1085 The Links Network Is Not The System That Reasons (E108)

Diagnosis issue [#1085](https://github.com/link-assistant/formal-ai/issues/1085);
case study in `docs/case-studies/issue-1085/`. Delivered items name the push
that landed them; the rest are in progress on the same pull request or are
sub-issues of #1085.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1085-1 | Name the Rust kernel and measure five shrink-only ceilings outside it (non-kernel lines, handler files, pending migrations, literal predicates, allowlist rows); a ceiling can move down and never up, and a release must show the line ceiling lower than at the previous tag. | Delivered: `data/meta/kernel-ratchet.lino`, `scripts/check-kernel-ratchet.rs`, gate `check_kernel_ratchet`, release rule in `self-development-status.yml`. The raisable handler-ledger ceilings are retired. |
| R1085-2 | Load seed and meta into the doublets store at startup and route matching through link queries; one generic interpreter for `when x do y` rules and `.lino` skills; migrate pending handlers smallest-first. | In progress on this pull request. |
| R1085-3 | Express the ladder's three edit shapes as link substitutions over the meta-language CST network with reconstruct, rustfmt, `cargo check`, `cargo test`; resolve requirement-shaped edits by link query over the self-AST census. | In progress on this pull request. |
| R1085-4 | Attribute self-authored commits by the model that produced them: `Formal-AI-Model` must be formal-ai and be named in the committed evidence; a session or model naming a hosted model is not self-authored. | Delivered: `scripts/self-hosting-attribution.rs::model_attribution`; `tests/unit/specification/self_hosting_metric.rs` fixtures carry the trailer. |
| R1085-5 | Count only behaviour-changing paths on both sides of the share; `docs/`, `dev/`, `experiments/`, `changelog.d/` and captured artifacts leave numerator and denominator. | Delivered: `is_non_authored_path` in `scripts/self-hosting-attribution.rs`. |
| R1085-6 | Record who opened each qualifying pull request in the ledger row. | Delivered: `self_authored_pull_request_author` rows, `FORMAL_AI_PULL_REQUEST_AUTHOR_LOOKUP=gh` in `release.yml`. |
| R1085-7 | Restate history under metric version 3 by appending rows, never rewriting; publish the figure. | Delivered mechanism: `--replay-epoch` (`scripts/self-hosting-replay.rs`); the restated rows are appended from the first CI run of the status workflow. |
| R1085-8 | Move the self-development floor off the release path onto an always-visible red-until-true status with no budget, window or bypass; releases cut on CI correctness alone. | Delivered: `.github/workflows/self-development-status.yml`; `release.yml` and `scripts/version-and-commit.rs` no longer gate; `tests/unit/ci-cd/issue_1014.rs`. |
| R1085-9 | Ladder leaves must compile; composites apply both children's diffs to one tree; depth 3 and above are requirement-shaped; the root is a real issue; run on pull requests with a ratcheted deepest passing level. | Delivered: `cargo check` per `.rs` leaf in `verify-node.sh` with a shared target directory. Test run, composite merge, requirement-shaped nodes: in progress. |
| R1085-10 | Cite the upstream benchmark row beside every curated 13/13 citation. | Delivered: `VISION.md`, `ROADMAP.md` (three sites). |
| R1085-11 | Failing upstream cases feed the learning cycle; the external-benchmarks workflow is red on a falling suite; explain the HumanEval task-0 transfer failure. | In progress on this pull request. |
| R1085-12 | The crates.io credential probe must not report a token rejected on a cookie-only endpoint; `/api/v1/me` is never called, and the read-only verdict is `unknown` with the reason. | Delivered: `scripts/preflight-credentials.sh`, `tests/unit/ci-cd/issue_1081/release_preflight.rs`. Run 34149311523 was the false positive. |
| R1085-13 | The macOS archive build budget covers the measured spread of the runner (11-26 min over ten `main` runs) with the cap at or under the 70% share rule. | Delivered: 1800 s budget, 55 m cap in `macos-core-tests.yml`. |
| R1085-14 | Work the open user-prompt frontier (#720, #721, #722, #724, #869, #1063, #447) by generalisation with four-language held-out tests. | Sub-issue of #1085 (D6). |
| R1085-15 | Move `dev/log` and raw case-study logs to an evidence store with a hashed Links Notation index; cap non-source additions per pull request; land #1072. | Sub-issue of #1085 (D7). |
| R1085-16 | Render status tables from ledgers, convert byte-equality pins to containment, require a justification for every CI gate, record a wall-clock ceiling. | Sub-issue of #1085 (D8). |
| R1085-17 | Finish or retire the traceability manual-confirmation column. | Sub-issue of #1085 (D9). |
