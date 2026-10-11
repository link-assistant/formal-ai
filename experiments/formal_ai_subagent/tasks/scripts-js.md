TASK (tags SCRIPTS-A, SCRIPTS-B): give the most-used `rust-script` gates a JavaScript twin, so nobody compiles Rust locally. JS-first parity applies to our own tooling too.

Why: `rust-script` compiles each script with cargo into `~/Library/Caches/rust-script`. That cache had grown to 3.7 GB on a nearly full disk. The user asked that no Rust be built locally. `experiments/formal_ai_subagent/local-gates.mjs` now runs `scripts/<name>.mjs` in place of `rust-script scripts/<name>.rs` when the twin exists, and skips the gate otherwise.

**SCRIPTS-A, the requirement pipeline:**
- `scripts/assemble-requirements.rs`
- `scripts/generate-requirement-status.rs`
- `scripts/render-status.rs`
- `scripts/check-requirement-status.rs`

**SCRIPTS-B, the checks agents hit most:**
- `scripts/check-file-size.rs`
- `scripts/check-debt-ratchet.rs`
- `scripts/check-hardcoded-language.rs`
- `scripts/check-minimal-core-boundary.rs`
- `scripts/check-worker-line-budget.rs`

For each script:
1. Write `scripts/<name>.mjs` with the same flags (`--write`, `--check`, `--base <ref>`, …), the same exit codes, and the same stdout/stderr lines, including the `::error` annotations. Node built-ins only. Files ≤1500 lines; shared helpers can go in `scripts/lib/*.mjs`.
2. **Prove parity without Rust.**
   - For the generators: `--check` (or a write to a temp dir) must reproduce the committed outputs byte for byte from the committed inputs: `docs/requirements/assembled/*`, `data/meta/requirement-status-ledger/*`, `docs/status.md`.
   - For the checks: they pass on the current tree. Make a deliberate small violation in a sandbox copy (`experiments/formal_ai_subagent/sandboxes/<tag>-*`), check that it fails with the same message the Rust script prints (read its source for the exact text), then remove the violation.
   - Add `node --test` cases in `rust/tests/web/scripts-js-twins.test.mjs` (SCRIPTS-A) or `rust/tests/web/scripts-js-twins-checks.test.mjs` (SCRIPTS-B).
3. **CI parity.** Add one small gate per twin under `data/meta/ci-gates/` (read an existing one for the format). In CI it runs both the Rust original and the JS twin and fails if their outputs or exit codes differ. The rust-script half runs only in CI.
4. Update the commands in `experiments/formal_ai_subagent/README.md` (Routine commands) and `preamble.md` to the `node scripts/<name>.mjs` form.
5. Write a ledger row in `docs/case-studies/pull-request-1188/formal-ai-dogfood.md`: SCRIPTS-A uses T300–T304, SCRIPTS-B uses T305–T309. Use Formal AI for the small edits (README lines, ledger rows, gate files): `node experiments/js_dogfood/drive.mjs --dir <dir> --steps 8 "<prompt>"`, with «» quotes. Log its failures in `gaps.md`.

Rules:
- **Never run rust-script or cargo locally.** Read the Rust sources; do not execute them.
- Do not change the Rust scripts' behaviour.
- Claim your files in `experiments/formal_ai_subagent/claims.md`.
- TEACH-F is editing `js/agentic` and its Rust twins; do not touch them.
- Do not commit.
- Report the files changed, the parity evidence per script, and anything that could not be made identical.
