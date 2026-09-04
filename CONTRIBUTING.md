# Contributing to formal-ai

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## How we develop Formal AI: drive the Agent CLI, never defer

**From issue #538 forward, this is the only way we develop the Formal AI
system.** We do not solve a task by editing code and data by hand and we do not
solve it partway and defer the rest to a roadmap. We solve it by **driving Formal
AI through its own [Agent CLI](https://github.com/link-assistant/agent)** (the
in-repo agentic driver in `src/agentic_coding/`, running against the
OpenAI-compatible `formal-ai serve` server), and we get *every* requirement done
in the same pull request.

This section is the development policy and target workflow, not a claim that
the repository is already autonomously self-coded. When an existing tool gap
forces a manual tool extension, record that boundary plainly, retry a smallest
leaf through Formal AI, and measure only genuinely session-authored lines. A
reviewed task decomposition must name its smallest leaves; the acceptance floor
is at least one real Formal-AI/Agent-CLI-authored leaf out of every five (20%),
with a captured session and paired commit trailers. Raise that measured share
over time; never relabel manual work as self-authored.

Concretely, every change must follow these rules:

1. **The tool authors the change, not you.** Drive the Agent CLI + Formal AI to
   produce the change. Where the output lands in the repo (e.g. seed data), a
   test must assert that the committed artifact is **byte-for-byte** what the
   Agent-CLI-driven recipe produces, so the tool — not a hand-edit — is the
   author and cannot silently regress. See the issue #538 case study
   ([`docs/case-studies/issue-538/`](docs/case-studies/issue-538/)) for the
   pattern and the committed `agent-cli-session*.json` sessions.
2. **No pre-emptive deferral, no refusals, no follow-ups.** "This is large or
   hard" is never a reason to ship a slice and route the rest to a roadmap. Find
   the smallest real, tested, reproducible slice of *each* requirement and
   execute it now, in this PR. Read
   [`docs/case-studies/issue-538/refusal-anti-pattern.md`](docs/case-studies/issue-538/refusal-anti-pattern.md)
   before opening a PR — it is the failed reasoning we do not repeat, and we do
   not teach Formal AI to refuse or defer like that.
3. **When the tool can't do it, extend the tool, then retry.** Falling back to a
   manual edit is allowed only after you have proven the Agent CLI / Formal AI
   cannot yet do it — and then you must immediately improve the Agent CLI /
   Formal AI so it *can* in general, and re-run through the tool.
4. **Prove generality with different words each time.** Use a *different* natural
   language request for each case so a passing run proves the solution is truly
   general, not hardcoded to one phrasing (issue #538 drives tomato and potato
   with two differently-worded requests).
5. **Report faithfully.** State what is done and verified plainly. Honesty means
   reporting results accurately; it is never a license to stop early or to dress
   a refusal as an "honest scope" section.
6. **Real Agent-CLI E2E tests in CI, plus a per-requirement test and a
   whole-task test.** Every change that touches the agentic path must add (or
   update) a real end-to-end test that boots `formal-ai serve` and drives it
   with the actual `@link-assistant/agent` CLI over the OpenAI-compatible
   endpoint — no mocks or in-process shortcuts. Keep the round-trip green in CI
   (see `test-agent-cli-e2e` in `.github/workflows/release.yml` and the driver
   script `experiments/agent_cli_e2e/run_agent_cli.sh`). In addition, ship one
   unit/integration test **per requirement in the issue** and one test that
   exercises the **whole task** end-to-end so a regression on any single
   requirement — or on the composition of all of them — breaks the build.
7. **Hardcoded cases only in tests; production code stays general.** A test may
   hardcode inputs and expected outputs (that is what a test is *for*), but the
   engine, planner, seed loader, and Agent-CLI-driven recipes never branch on a
   specific concept, phrase, or URL. If the only way to make a green case pass
   is a match-on-literal in `src/`, extend the general routing table (`concept
   registry`, `capability classifier`, `plan_chat_step`) so future concepts get
   the same treatment for free.
8. **Real logs in the case study, not synthesized ones.** When a case study
   claims the Agent CLI drove the change, it must ship the real captured log of
   the round-trip (see `docs/case-studies/issue-538/agent-cli-e2e-run.log`), and
   the committed session JSON must be reproducible byte-for-byte by
   `cargo test`.
9. **Commit in small, atomic steps.** Every commit should be independently
   useful and reviewable — one logical change per commit, buildable in
   isolation. Interrupted work stays preserved in the PR because each commit
   already stands on its own; do not batch a day of unrelated edits into one
   commit.

### Testing external agentic CLIs

When validating `codex`, `opencode`, `gemini`, or the `agent` CLI against a
local Formal AI server, follow
[`docs/testing/agentic-cli-tools.md`](docs/testing/agentic-cli-tools.md). The
guide defines the fixture markers, logging proxy assertions, phrasing matrices,
and CI shape needed to prove that a result came from Formal AI and that the
client actually executed the expected tools.

### Spawn a reference assistant when you need a natural-language target

When a change is about *how Formal AI talks* to the user — narration before a
tool runs, an error explanation, the wording of a question — it helps to see how
a strong conversational assistant would phrase the same step. Spinning up a free
model in `claude`, `codex`, or `opencode` on the *identical* prompt and reading
its reply gives a concrete, natural target to match, and is a recommended way to
calibrate tone before writing the seed catalog text and its tests. Treat those
transcripts as reference examples only: the phrasing you ship still lives in the
`.lino` seed data (never hardcoded in the solver), and every user-visible string
must be asserted by a test. Issue #819's narration rewrite was tuned this way —
the desirable "Let me look on your Desktop for …" shape came from comparing
Formal AI's output against `claude` and `codex` on the reported request.

### Replaying the self-coding loop

```bash
cargo build --release --bin formal-ai
examples/self-coding/run.sh
cargo test self_coding_session_replays
```

For a real GitHub issue, run `examples/self-coding/run.sh --live ISSUE_URL`,
which invokes the command below — Hive Mind drives the Agent CLI, which drives
the local Formal AI server:

```bash
solve ISSUE_URL --tool agent --model formal-ai --attach-logs --verbose
```

### Always run automated `solve` sessions with `--attach-logs --verbose`

Every automated session started against this repository — by hand or by a
maintainer's dispatcher — must carry both flags:

```bash
solve https://github.com/link-assistant/formal-ai/issues/905 \
  --tool agent --model formal-ai --attach-logs --verbose
```

Both are load-bearing, and neither substitutes for the other:

- **`--attach-logs`** publishes the session log to the pull request, so a
  failure leaves behind evidence rather than a single line in a comment.
- **`--verbose`** is what makes the Agent adapter dump the **raw JSON** of every
  error record and every fatal startup log record
  ([link-assistant/hive-mind#2143](https://github.com/link-assistant/hive-mind/pull/2143)).
  That raw record is what survives a future payload shape the renderer does not
  know about.

This is not a style preference; it is a precondition of the self/auto-learning
loop this repository is built around. On 2026-08-04 a run on PR #927 failed
after 22 seconds and recorded its whole reason as `[object Object]`, with no log
attached — a failure that is unlearnable by construction, because the next
iteration has nothing to act on. The container is gone and that cause is
unrecoverable. The full timeline, root causes, and raw evidence are in
[`docs/case-studies/issue-973/README.md`](docs/case-studies/issue-973/README.md).

`tests/issue_973_solve_flags.rs` enforces the policy: it scans the guides and
scripts this repository publishes and fails when any `solve` invocation drops
either flag. Recorded history under `docs/case-studies/`, `dev/log/`, and
`experiments/` is exempt — a past run stays as it happened.

### Recording self-authorship

The release metric counts a commit as Formal AI-authored only when its commit
message records the first two trailers below. A contribution intended to
satisfy the per-release self-development floor must record all three:

```text
Formal-AI-Session: <session-id>
Formal-AI-Evidence: <repo-relative committed evidence path>
Formal-AI-Pull-Request: https://github.com/<owner>/<repo>/pull/<number>
```

The evidence path must exist in that commit. It may name one evidence file or a
directory bundle; one file at or below the path must contain both `formal-ai`
and the exact session id. Add one pair per session when multiple sessions
authored a commit. Do not add these trailers to a human-authored or manually
corrected commit. The pull-request reference counts toward the release floor
only when Git history proves that the same commit object reached the matching
GitHub merge commit; a direct commit carrying a claimed PR URL does not count.
Every non-merge commit introduced by that pull request must satisfy the same
session, evidence, and pull-request checks. One attributed commit cannot make a
mixed manually authored pull request count as end-to-end self-development.

Every release cycle must contain at least one such merged contribution. It goes
through the ordinary pull-request review, CI, and promotion policy without an
AI-specific bypass. The next release's target carries forward from the previous
row and rises with the measured trailing share, so it must not decrease on its
own. It moves down only when a reviewed commit writes a
`target_override_basis_points` onto the newest comparable row of
`data/meta/self-hosting-ledger.lino`, which replaces the ratchet and carries
forward until another commit changes or removes it. That is the only lever:
there is no flag, environment variable, or workflow input that moves the target,
so every change to the level is visible in a diff and named in the release
notes. If the floor or target is missing, merge more
reviewed Formal AI-authored work before retrying the release. The metric counts additions plus deletions from non-merge
commits and ignores binary files. Reproduce it with
`rust-script scripts/self-hosting-metric.rs --since <previous-tag>`.

Before checking the pull-request ratchet locally, fetch annotated release tags
with `git fetch origin --tags`; without the latest tag, the check reports a skip.

## Contribution rights and external material

By intentionally submitting a contribution to this repository, you represent
that you have the authority to submit it and offer your copyrightable
contribution under the repository's [Unlicense](LICENSE), including its
public-domain dedication and permissive fallback terms. If you do not own the
rights or cannot make that offer, do not submit the material.

Third-party material remains subject to its own license and terms. Identify the
source, exact revision, license, required notices, and any naming or use
conditions in the pull request and repository provenance record. Public access,
zero price, AI generation, or appearance in an issue does not remove those
conditions. Follow [LEGAL-COMPLIANCE.md](LEGAL-COMPLIANCE.md) and complete
[`docs/legal/source-review.md`](docs/legal/source-review.md) before using any
external material for training or distillation.

Never paste or attach any of the following to issues, pull requests,
discussions, logs, fixtures, or commits:

- leaked material or proprietary source code;
- a paid or access-controlled dataset without redistribution permission;
- a large verbatim copyrighted work or bulk model output;
- credentials, private keys, confidential information, or trade secrets; or
- real personal data.

For debugging or critique, use the smallest lawful excerpt, link to the
authorized source, record provenance, and redact unrelated content. Maintainers
may remove, redact, or quarantine suspect material and its downstream copies
while a rights, privacy, safety, or Terms-of-Service concern is investigated.
Removal from a public thread is not approval to retain another copy.

## Development Setup

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/YOUR-USERNAME/formal-ai.git
   cd formal-ai
   ```

2. **Install Rust**

   Install Rust using rustup (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Install development tools**

   ```bash
   rustup component add rustfmt clippy
   cargo install rust-script
   ```

4. **Install pre-commit hooks** (optional but recommended)

   ```bash
   pip install pre-commit
   pre-commit install
   ```

5. **Build the project**

   ```bash
   cargo build
   ```

## Development Workflow

1. **Create a feature branch**

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes**

   - Write code following the project's style guidelines
   - Add tests for any new functionality
   - Update documentation as needed

3. **Run quality checks**

   ```bash
   # Format code
   cargo fmt

   # Lint executable test targets; compile-check examples without linking 100+ binaries
   cargo clippy --lib --bins --tests --all-features
   cargo check --examples --all-features

   # Check file sizes (requires rust-script)
   rust-script scripts/check-file-size.rs

   # Check for hardcoded natural language in src/ (R379, requires rust-script)
   rust-script scripts/check-hardcoded-language.rs

   # Run all checks together
   cargo fmt --check && cargo clippy --lib --bins --tests --all-features && cargo check --examples --all-features && rust-script scripts/check-file-size.rs && rust-script scripts/check-hardcoded-language.rs
   ```

4. **Run tests**

   ```bash
   # Run all tests
   cargo test

   # Run tests with verbose output
   cargo test --verbose

   # Run doc tests
   cargo test --doc

   # Run a specific test
   cargo test test_name

   # Run the browser unit suite (the site's production JavaScript)
   npm run test:web
   ```

   CI caps each test-matrix job at 10 minutes. Rust's built-in `cargo test` runner does not provide a portable global per-test timeout, so wrap long-running network, IO, or async tests with explicit test-level deadlines. If a repository adopts `cargo nextest`, configure runner deadlines with options such as `--slow-timeout` and `--leak-timeout`.

5. **Add a changelog fragment**

   For any user-facing changes, create a changelog fragment:

   ```bash
   # Create a new file in changelog.d/
   # Format: YYYYMMDD_HHMMSS_description.md
   touch changelog.d/$(date +%Y%m%d_%H%M%S)_my_change.md
   ```

   Edit the file to document your changes:

   ```markdown
   ### Added
   - Description of new feature

   ### Fixed
   - Description of bug fix
   ```

   **Why fragments?** This prevents merge conflicts in CHANGELOG.md when multiple PRs are open simultaneously.

6. **Commit your changes**

   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

   Pre-commit hooks will automatically run and check your code.

7. **Push and create a Pull Request**

   ```bash
   git push origin feature/my-feature
   ```

   Then create a Pull Request on GitHub.

## Code Style Guidelines

This project uses:

- **rustfmt** for code formatting
- **Clippy** for linting, with `all`/`pedantic`/`nursery` enabled in `Cargo.toml`'s `[lints.clippy]` (a few noisy lints are explicitly allowed there)
- **cargo test** for testing

### Code Standards

- Follow Rust idioms and best practices
- Use documentation comments (`///`) for all public APIs
- Write tests for all new functionality
- Keep functions focused and reasonably sized
- Keep Rust files under 1000 lines (`.lino` files and the browser worker JavaScript are capped at 1500); all limits are enforced by `rust-script scripts/check-file-size.rs`
- Use meaningful variable and function names

### Documentation Format

Use Rust documentation comments:

```rust
/// Brief description of the function.
///
/// Longer description if needed.
///
/// # Arguments
///
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of when errors are returned
///
/// # Examples
///
/// ```
/// use my_package::example_function;
/// let result = example_function(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn example_function(arg1: i32, arg2: i32) -> i32 {
    arg1 + arg2
}
```

## Testing Guidelines

- Run the suite through `scripts/cargo-test.sh` rather than `cargo test`
  directly. It takes the same arguments and adds two things a workstation
  needs:

  ```bash
  scripts/cargo-test.sh                          # whole suite
  scripts/cargo-test.sh --test unit issue_907    # one module
  ```

  **macOS runs platform tests, not the whole suite.** No code in `src/` branches
  on macOS versus Linux — all eight conditionals are `cfg(unix)`, true on both —
  so pure Rust logic cannot behave differently there. Every macOS-only failure
  this repository has recorded came from the environment instead: GNU coreutils
  absent (`timeout`), bash 3.2 without `mapfile`, subprocess and path handling.

  Running everything twice cost real time: 2895 tests moved a 916 MB archive to
  each of eight runners, 7 GB per run, and two of those downloads failed
  outright. The macOS lane now runs the 139 tests whose behaviour can differ —
  about ten seconds on one runner.

  **When something does behave differently on macOS, add its module to
  `data/meta/macos-platform-tests.lino`** with a line saying what differs. Do
  not widen the filter back to everything; the file is the list of what macOS is
  actually for, and `issue_1017::the_macos_lane_selects_a_non_empty_set_of_tests`
  fails if it empties out.

  **Start the longest work first.** Whenever work is split across parallel
  workers -- test partitions, matrix legs, anything fanned out -- the long tasks
  must be scheduled first and the short ones packed in behind them. A long task
  started last runs alone on a machine everything else has already finished
  waiting for, and that tail is pure serial time on the critical path.

  This is longest-processing-time-first, and it is not a preference: measured on
  run 32591020809 (2895 tests, 4704s of work over eight partitions), splitting
  by test index gave a **870s** worst partition against a **588s** ideal, while
  LPT hit 588s exactly. 282s of the critical path, spent idling.

  `scripts/plan-test-partition.rs` implements it for the macOS test slices from
  the durations recorded in `data/meta/test-durations.lino`, and the
  `check_test_partition_balance` CI gate fails when the plan drifts out of
  balance -- so a regression back to index order cannot land quietly. Apply the
  same rule to any new fan-out: sort by cost, descending, before assigning.

  **Use the whole machine on CI.** An ephemeral runner is billed for the minutes
  it is alive, so leaving cores idle only makes the wait longer. Do not add
  `max-parallel`, and do not cap test threads on CI; the local half-CPU cap
  below exists for a shared laptop, not for a runner.

  **Half the CPUs locally, all of them on CI.** A bare `cargo test` starts one
  compile job *and* one test thread per core, which pins the whole machine for
  the length of the run. The wrapper caps both at half the cores unless `CI` is
  set, so an ephemeral runner still uses everything it is paying for. Override
  with `CARGO_TEST_JOBS=<n>`.

  **The cache keeps one build, not every build.** Cargo never removes anything,
  so `target/` accumulates artifacts from every branch and dependency version
  and reaches several gigabytes within days. The wrapper prunes to the artifacts
  the latest build produced, and CI does the same after its test step so the
  saved actions cache carries one build rather than a growing pile. Run
  `scripts/prune-build-cache.sh` on its own to reclaim space at any time, or set
  `CARGO_TEST_NO_PRUNE=1` to keep everything for a debugging session.

  **Every commit sweeps.** The `prune-build-cache` pre-commit hook runs on every
  commit, not only Rust ones -- a docs-only commit leaves the previous build's
  artifacts on disk just the same. Disk is reclaimed as a matter of course
  rather than when someone remembers.

  ```bash
  cargo install cargo-sweep   # strongly recommended
  ```

  With cargo-sweep installed the pruner asks cargo *which artifacts the current
  build actually references* and removes the rest, so a dependency the next
  build still needs survives even if it was compiled weeks ago. Without it the
  pruner falls back to comparing modification times, which cannot tell a stale
  artifact from a current one that simply did not need rebuilding -- it deletes
  live dependencies and the next build recompiles them. Both keep the cache
  small; only cargo-sweep keeps it *useful*.

  `CARGO_TARGET_MAX_SIZE_MB` caps the tree after the sweep, defaulting to 4096
  (4GB) locally and to no ceiling on CI, where the runner is billed for the
  rebuild rather than the disk.

- Write tests for all new features
- Maintain or improve test coverage — this is enforced, not requested. CI
  measures two separate denominators, Rust and browser, against the reviewed
  floors in `coverage/baseline.json` and fails on a decrease. Raising a floor is
  a one-liner; lowering one requires an explicit reviewed justification recorded
  in the file. See [docs/design/coverage-ratchet.md](docs/design/coverage-ratchet.md).

  ```bash
  # Browser denominator
  npm run coverage:web && npm run coverage:ratchet -- --only browser

  # Rust denominator
  cargo llvm-cov --all-features --lcov --output-path lcov.info
  npm run coverage:ratchet -- --only rust

  # Lock in a gain
  npm run coverage:ratchet -- --only browser --update-baseline
  ```
- Use descriptive test names
- Organize tests in modules when appropriate
- Use `#[cfg(test)]` for test-only code

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod my_feature_tests {
        use super::*;

        #[test]
        fn test_basic_functionality() {
            assert_eq!(my_function(), expected_result);
        }

        #[test]
        fn test_edge_case() {
            assert_eq!(my_function(edge_case_input), expected_result);
        }
    }
}
```

## Project Conventions (recurring maintainer recommendations)

These conventions recur in almost every issue review. They are collected here so
contributors — human and AI — can apply them up front instead of rediscovering
them in review. They reflect the project's vision: a deterministic, symbolic
agent whose every answer is a projection of an append-only event log, with no
hardcoded prompt→answer tables.

1. **Mirror parity (Rust ↔ JS worker).** Every reasoning path in the Rust engine
   (`src/*.rs`) has a twin in the browser worker (`src/web/formal_ai_worker.js`
   loader plus the `src/web/worker/formal_ai_worker_*.js` shards),
   so the CLI, library, HTTP server, Telegram bot, and website all answer the
   same prompt identically. A behavioural change in one **must** be mirrored in
   the other in the same PR. Name and comment the twin so the parity is obvious
   (e.g. "Mirrors `try_x` in `src/solver_handler_x.rs`").
   Mirror parity is the transitional contract while JS worker logic remains:
   under the compiled-logic doctrine (REQUIREMENTS.md R536), prefer absorbing
   the path into the Rust→WASM worker over adding a new JS twin, and never
   grow the worker line budget (`scripts/check-worker-line-budget.rs`).

2. **Data-driven seed, no hardcoded natural language in code (issues #386,
   #513).** Natural language is *data*, never a string literal in the engine.
   This applies to **both directions** of every reasoning path:
   - **Triggers / detection.** All multilingual phrases, surfaces, run verbs,
     shell tokens, concept summaries, and the tool registry live in
     `data/seed/*.lino`. Recognisers ask the lexicon for a *meaning* by role
     (`lexicon().meanings_with_role(ROLE_…)`) or load a named vocabulary
     (e.g. `seed::terminal_command_vocabulary()`); they never hardcode
     per-language phrase arrays or branch on literal user phrasings.
   - **Responses / output.** Every user-facing answer string is a template in
     `data/seed/multilingual-responses.lino` looked up by intent
     (`seed::response_for(intent, lang)` in Rust, `answerFor(...)` in the
     worker). Code fills placeholders like `{command}`; it does not embed the
     surrounding prose.
   - **Web front-end (React).** Every string the user sees in `src/web/app.js`
     — permission-panel titles, button labels, status words, onboarding copy,
     system messages — is a catalog entry in `src/web/i18n-catalog.lino`, looked
     up at render time via `t("<key>", params)` (the `window.FormalAiI18n`
     engine). Never pass a prose string literal as a child of `h(...)`; route it
     through `t(...)` so it follows the active UI language and fills placeholders
     like `{granted}/{total}`.

   The principle is **meanings ↔ naturalization**: a *meaning* (a slug grounded
   in seed data) can be *naturalized* into a natural-language surface, and any
   natural-language word can be *formalized* back into a meaning. Code only ever
   moves meanings around; the words live in the seed. Add a new cue or answer by
   editing the `.lino` file and declaring the role/intent — not by typing a
   phrase into `src/*.rs` or `formal_ai_worker.js`.

   This is enforced by CI, not just convention:
   - **Total reference-closure gate** (`tests/unit/total_closure.rs` →
     `scripts/audit-total-closure.py`, run by `cargo test --tests`). Every bare
     value token in any `data/seed/*.lino` must resolve to a defined meaning, a
     declared role, a cached dictionary lemma, or a Wikidata id. New vocabulary
     that resolves to nothing fails the build. Ground new tokens by running
     `python3 scripts/close-total.py` (idempotent; emits each unresolved token
     as a first-class meaning under `data/seed/closure-generated-*.lino`) until
     `python3 scripts/audit-total-closure.py` reports `unresolved_distinct: 0`.
     Those shards are **content-addressed**: each meaning lands in the shard
     picked by `sha256(slug) % SHARD_COUNT`, so a new token rewrites exactly one
     file and leaves the others byte-identical. Do not re-sort them into
     alphabetical files — filling shards sequentially makes every shard depend
     on the size of everything before it, which is what made `data/seed`
     conflict in almost every pull request. `SHARD_COUNT` is fixed rather than
     derived from the corpus, because changing it reshuffles every shard once;
     raise it only when a shard approaches the 1500-line data-file limit.
     `./experiments/issue-909-seed-shard-conflict-blast-radius.sh` asserts the
     property by adding one token and counting the files it dirties.
   - **Worker seed parity checks.** Where the JS worker consumes a generated web
     seed copy, a `--check` guard fails the build on loader regressions and on
     drift in a present mirror (e.g. the “Check terminal vocabulary worker seed
     wiring” CI step runs
     `node experiments/issue-513-sync-worker-terminal.mjs --check`). Refresh
     the web seed copy with `scripts/sync-seed.sh` or by running the same script
     without `--check`.
   - **Web-UI hardcoded-string guard (#511).** `npm --prefix tests/e2e run
     check:web-hardcoded-ui` parses every `h(...)` call in `src/web/app.js` and
     fails the build when a child argument is a bare prose string literal, so new
     English text cannot leak into the UI. `npm --prefix tests/e2e run check:i18n`
     asserts every required key exists in all four locales and that sample
     interpolations render. When you add a web-UI string: add the key + all four
     translations in `src/web/i18n-catalog.lino`, register it in `REQUIRED_KEYS`
     in `tests/e2e/scripts/check-i18n-catalog.mjs`, and render it with `t(...)`.

   See `docs/design/no-hardcoded-natural-language.md` for the full rationale,
   the meanings ↔ naturalization model, and a worked example.

3. **Roles are declared, then generated.** When you add a meaning with a new
   `role`, declare it as a `ROLE_*` constant in `src/seed/roles/*.rs`, re-export
   it from `src/seed.rs`, and regenerate the registry with
   `python3 scripts/generate-role-registry.py` (keeps `data/seed/roles.lino` in
   lockstep; enforced by `reference_closure` tests).

4. **Supported-language coverage.** New conversational cues should cover the
   project's supported languages (currently en, ru, hi, zh). The
   `tests/e2e/scripts/check-*.mjs` guards fail a one-language change.
   Translation changes have the stricter issue #526 rule: add or update
   round-trip tests that prove language-to-meta-to-same-language survival and
   every supported language-pair path through the meta language. Code
   translation changes must preserve a `meaning:` link across the source ->
   target -> source round trip (for example Rust <-> JavaScript), not just render
   plausible syntax.

5. **Fix everywhere, not just the reported spot.** If a defect has more than one
   site (most do, because of mirror parity), fix all of them in the one PR.

6. **Reproduce first, then fix.** Add a failing test that reproduces the issue
   before implementing the fix; a bug fix without a reproducing regression test
   is treated as incomplete.

7. **When data is insufficient, add tracing.** If there is not enough signal to
   find a root cause, add debug output / a verbose mode (default **off**) and
   keep it in the code so the next iteration has the data.

8. **Case study per issue.** Download the issue's logs and data into
   `docs/case-studies/issue-{id}/` (raw JSON under `raw-data/`) and write a
   `README.md` that reconstructs the timeline, enumerates every requirement,
   finds the root cause(s), surveys prior art / existing libraries, and records
   the implemented fix and its verification.

9. **Report upstream when relevant.** If an issue is rooted in another
   repository we can file against, open an issue there with a reproducible
   example, a workaround, and a suggested fix.

10. **One PR per issue.** Plan and execute everything for an issue in a single
    pull request; commit atomic, individually useful steps so interrupted work
    stays preserved.

11. **Prefer the meta algorithm; drive Formal AI to solve its own tasks
    (direction set by issue #538).** The long-term way we develop this project is
    to treat every task as a message formalized into the meta language and to let
    Formal AI — driven through its own Agent CLI
    (<https://github.com/link-assistant/agent>) — reason about and solve the task
    by editing its own data (memory) and meta algorithm (reasoning), rather than
    a human hand-coding each answer. When Formal AI cannot yet perform a step,
    the goal is to improve the meta algorithm just enough that it can, verifying
    generality by phrasing the same request different ways and by reproducing the
    change in a clean repository copy driven by the Agent CLI.

    **Honest current status.** This is not aspirational: from issue #538 forward
    it is how changes are produced (see the top-of-file rules 1–5). The Agent CLI
    drives real, byte-for-byte-reproducible changes today — the self-hosting loop
    (spawn a Formal AI server, hand it the task, capture the Agent-CLI session JSON
    that reproduces the change) is what wrote the #538 seed data, the potato
    enrichment, and the generated recipe diagrams; `scripts/reproduce-issue-538.sh`
    regenerates all three on a clean checkout. What is honestly *not yet built* is
    autonomous handling of the sweeping, open-ended axes (e.g. a full CST/AST-in-
    data round trip, or one-shot handling of an arbitrary unseen issue). Those are
    **not** parked on a roadmap: each names its smallest real, testable next slice
    in [`docs/case-studies/issue-538/requirements.md`](docs/case-studies/issue-538/requirements.md)
    and is executed the same way — by extending Formal AI / the Agent CLI until the
    tool can do it and a test goes red on regression (rule 3), never by hand-editing
    and deferring. When a task's requirements genuinely conflict (as issue #538's
    small concrete ask sits inside a much larger vision), surface the contradiction
    explicitly and still deliver a real, tested slice of *each* axis — never dress
    "did part of everything" as an honest scope cut.

12. **Link the issue with a GitHub closing keyword; never "Addresses"
    (issue #960).** A pull-request description must close its issue with one of
    GitHub's [recognised keywords](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue)
    — write `Fixes #146` or, preferably, the full form
    `Fixes https://github.com/link-assistant/formal-ai/issues/146`. Words like
    *Addresses*, *Relates to*, *Part of* and *Refs* read to a human exactly like
    a link and to GitHub like plain prose: the issue silently stays open after
    the merge. `scripts/check-pull-request-link.rs` reads the description
    (`PR_BODY`, or a file passed as its argument) and fails the build on a
    missing closing keyword or a non-closing word used in its place.

13. **Case study per pull request.** Alongside the per-issue case study
    (rule 8), a case study that documents a *pull request* — its review
    conversation, CI history, and the decisions taken in it — lives in
    `docs/case-studies/pull-request-{id}/`, mirroring the `issue-{id}` layout
    (raw JSON under `raw-data/`). Keep issue-driven and PR-driven narratives in
    their own directories so neither is buried inside the other.

14. **The 1500-line Links Notation cap covers cached data too (issue #960).**
    `scripts/check-file-size.rs` and `tests/unit/data_files.rs` apply the cap to
    every `.lino` file in the repository, `data/cache/wikidata/` included.
    Generated-but-committed is not a reason to be exempt; it is the reason to be
    measured, because the generator is what can breach the cap. When a fetched
    response would exceed it, split it into `<bucket>-partN.lino` the way
    `examples/refresh_translation_cache.rs` already does.

15. **Budget cache buckets: 128 records each (issue #960).**
    `MAX_SEED_RECORDS_PER_BUCKET` in `src/translation/cache.rs` is enforced, not
    merely recorded: `scripts/check-cache-budget.rs` fails when a bucket under
    `data/cache/` holds more than 128 records (a record is a file stem, so
    `Q1860.json` and `Q1860.lino` count once). Bucket or trim instead of growing
    past the cap. The three buckets whose size is *forced* by the total-closure
    gate — `wikidata/entity`, `wordnet/en`, `wiktionary/en`, where every record
    exists because a seed token references it — are listed as
    `CLOSURE_DRIVEN_BUCKETS` with a written reason, and pay for the exemption
    with a stricter invariant: the check fails if any of their records is an
    orphan nothing references.

16. **Tests are documentation: assert the exact answer (issue #960).** A
    behavioural or conversational test should show the reader what the system
    says, not merely that the reply is not empty. Assert the answer verbatim
    (`assert_eq!(response.answer, "…")`, or membership in an explicit list of
    exact answers) and keep looser `contains` guards only as extra checks
    *after* the exact one. `scripts/check-tests-as-docs.rs` enforces this as a
    burn-down ratchet: existing loose-only tests are recorded in
    `scripts/tests-as-docs-allowlist.txt`, new ones fail the build, and a row
    that has been made explicit must be pruned (`--write` regenerates the list).

## Merge conflicts are a layout bug (issue #991)

`python3 scripts/analyze-merge-conflicts.py` replays every merge in this
repository's history with `git merge-tree` and counts what git could not merge on
its own. Of 1914 conflict-resolution events across 884 merges, only 37.4% were
two people changing the same behaviour. The rest conflicted because of *where the
content sat*: an appended list entry, a regenerated artifact, a numbered file
name. Those are bugs in the layout, and the layout is what we fix.

The measurement is in
[`docs/case-studies/issue-991/merge-conflict-analysis.md`](docs/case-studies/issue-991/merge-conflict-analysis.md),
the mechanisms are declared in
[`data/meta/merge-conflict-policy.lino`](data/meta/merge-conflict-policy.lino),
and `rust-script scripts/check-merge-conflict-policy.rs` fails the build when a
path that has actually been conflicting is neither mechanized nor deferred with
a written reason. **You never need to run `git config` for any of this**: every
mechanism uses git's built-in `merge=union` driver or a committed generator, so a
fresh clone is already conflict-proof.

**What to do when you add something.**

| You are adding | Do this | Do *not* |
| --- | --- | --- |
| a Rust module or test module | append the `mod` line anywhere in the list file; `rust-script scripts/normalize-ordered-lists.rs --write` sorts it | edit the list by hand to keep it sorted — the union driver will reorder it anyway |
| a CI check | write one file in `data/meta/ci-gates/`, named after the check | add a `- name:`/`- run:` step to `.github/workflows/release.yml` |
| a `data/seed/*.lino` file | add one `seed <name>` entry to `data/meta/seed-registry.lino` and run `rust-script scripts/generate-seed-registry.rs --write` | edit `src/seed/embedded.rs`, `src/seed/embedded_registry.rs`, `src/web/seed-files.js`, or the loader's file list |
| a requirement | write `docs/requirements/issue-NNNN-*.md` and run `rust-script scripts/assemble-requirements.rs --write` | append a section to `REQUIREMENTS.md` |
| a worker module | name the file after its subject (`formal_ai_worker_<subject>.js`) | claim the next free number |
| generated data | commit it and register its regenerate + verify commands in the policy | leave it un-verified: a union merge of an artifact lands silently |

**The two rules behind the table.**

1. **A list belongs in a file of its own.** A list that shares a file with logic
   cannot be union merged, because a union of two logic edits can compile and
   still be wrong. Move the list to a sibling file that contains nothing else —
   `modules.rs`, `worker-modules.js`, `seed-registry.lino` — and union merge only
   that file. Adding an item then stops being an edit anyone else can collide
   with.
2. **Every union-merged file has a verifier.** A union never blocks a merge, so
   without a checker a stale or duplicated union lands silently. Each
   union-merged path registers a `verify` command that fails while the unioned
   result is not canonical, or declares `union_is_terminal true` because every
   possible union of it is already correct content (only `.gitkeep` qualifies).

**Regenerating everything after a merge.** `bash
scripts/regenerate-derived-artifacts.sh` runs every registered generator in one
pass; run it after resolving a merge and commit whatever it changes.

If you hit a conflict that none of this covers, that is data. Add the path to the
policy — as a mechanism if it has a shape, or as a `deferred` entry with an
honest reason if the collision is genuinely semantic. A deferral with a stated
reason is a decision; an uncovered path is an omission, and CI treats them
differently.

## Pull Request Process

1. Ensure all tests pass locally
2. Update documentation if needed
3. Add a changelog fragment (see step 5 in Development Workflow)
4. Ensure the PR description clearly describes the changes
5. Link the issue the PR closes with a GitHub closing keyword — `Fixes #146` or
   `Fixes https://github.com/link-assistant/formal-ai/issues/146`. **Never**
   write `Addresses #146`, `Relates to #146`, or `Part of #146` in its place:
   GitHub does not recognise those, so the issue stays open after the merge.
   Verify locally before opening the PR:

   ```bash
   PR_BODY="$(gh pr view <number> --json body --jq .body)" \
     rust-script scripts/check-pull-request-link.rs
   ```

   CI runs the same check on every pull request.
6. Put any case study *about the pull request itself* in
   `docs/case-studies/pull-request-{id}/` (issue case studies stay in
   `docs/case-studies/issue-{id}/`)
7. Wait for CI checks to pass
8. Address any review feedback

## Changelog Management

This project uses a fragment-based changelog system similar to [Scriv](https://scriv.readthedocs.io/) (Python) and [Changesets](https://github.com/changesets/changesets) (JavaScript).

### Creating a Fragment

```bash
# Create a new fragment with timestamp
touch changelog.d/$(date +%Y%m%d_%H%M%S)_description.md
```

### Fragment Categories

Use these categories in your fragments:

- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Features that will be removed in future
- **Removed**: Features that were removed
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### During Release

Fragments are automatically collected into CHANGELOG.md during the release process. The release workflow:

1. Collects all fragments
2. Updates CHANGELOG.md with the new version entry
3. Removes processed fragment files
4. Bumps the version in Cargo.toml
5. Creates a git tag and GitHub release

## Project Structure

```
.
├── .github/workflows/    # GitHub Actions CI/CD
├── analysis/             # Ad-hoc analysis notes
├── changelog.d/          # Changelog fragments
│   ├── README.md         # Fragment instructions
│   └── *.md              # Individual changelog fragments
├── data/                 # Canonical knowledge surface (Links Notation)
│   ├── seed/             # Seed data every interface reads (see "Data is the interface")
│   ├── meta/             # Grounded meta-algorithm recipes and lexicons
│   ├── benchmarks/       # Benchmark suites and license provenance
│   ├── cache/            # Precached external sources (Wikidata, …)
│   └── parity/           # Cross-runtime parity fixtures
├── desktop/              # Electron desktop shell
├── docs/                 # Case studies, design notes, diagrams, guides
├── examples/             # Usage examples
├── experiments/          # Verification harnesses and one-off experiments
├── scripts/              # Rust scripts (via rust-script) and installers
├── src/
│   ├── lib.rs            # Library entry point
│   ├── main.rs           # Binary entry point
│   ├── solver.rs         # UniversalSolver — the 11-step loop
│   ├── solver_handlers/  # Handler family invoked through the method registry
│   ├── agentic_coding/   # Agentic-CLI planner, recipes, and driver
│   ├── proof_engine/     # Decision procedures
│   ├── summarization/    # Formalize → summarize → deformalize pipeline
│   ├── translation/      # Formalization and translation through meanings
│   ├── seed/             # Seed loading, lexicon, and role constants
│   └── web/              # Browser demo: UI, worker/, and wasm-worker/
├── tests/                # Unit, integration, source-mirror, and e2e tests
├── vscode/               # VS Code extension (desktop and web)
├── .gitignore            # Git ignore patterns
├── .pre-commit-config.yaml  # Pre-commit hooks
├── ARCHITECTURE.md       # How the implemented pipeline is wired
├── build.rs              # Build script
├── Cargo.toml            # Project configuration
├── CHANGELOG.md          # Project changelog
├── compose.yaml          # Docker Compose profiles
├── CONTRIBUTING.md       # This file
├── Dockerfile            # Docker-in-Docker Telegram runtime
├── GOALS.md              # What counts as success
├── LICENSE               # Unlicense (public domain)
├── NON-GOALS.md          # What we explicitly do not build
├── package.json          # Web/desktop/VS Code tooling scripts
├── README.md             # Project README
├── REQUIREMENTS.md       # Issue-by-issue requirement matrix
├── ROADMAP.md            # Requirement-level implementation status
└── VISION.md             # Values and long-term direction
```

## Release Process

This project uses semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Releases are managed through GitHub releases. To trigger a release:

1. Manually trigger the release workflow with a version bump type
2. Or: Update the version in Cargo.toml and push to main

## Getting Help

- Open an issue for bugs or feature requests
- Use discussions for questions and general help
- Check existing issues and PRs before creating new ones

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

Thank you for contributing!
