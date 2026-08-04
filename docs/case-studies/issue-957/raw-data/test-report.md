# formal-ai verification battery — 2026-08-04

Repo: `/Users/konard/Code/Archive/link-assistant/formal-ai` @ `de61602f` (main, v0.326.0)

## Environment

- macOS 15.7.7 (Darwin 24.6.0, arm64), zsh; system bash is **3.2.57** (relevant below)
- rustc/cargo 1.96.1, bun 1.2.20, node v20.19.4, npm 11.7.0
- Warm `target/` cache (8.5 GB) present before the run
- No source file was modified. `bun run build:web` rewrote the 4 committed bundle files; they were restored with `git checkout` (see step 5). Working tree at the end: only the two pre-existing untracked files `formal-ai-harness-latest.lino`, `formal-ai-server-latest.lino`.

## Step results

### 1. CI-gate discovery — DONE
Main pipeline is `.github/workflows/release.yml`: `cargo fmt --all -- --check`; `cargo clippy --lib --bins --tests --all-features -- -D warnings`; `cargo check --examples --all-features`; `cargo doc --no-deps --lib`; `cargo metadata --locked`; `cargo test --lib --bins --tests --all-features`; `cargo test --doc`; `bun install --frozen-lockfile && bun run build:web`; `npm ci --prefix tests/e2e` + 6 static checks; desktop/vscode smokes; seed sync via `scripts/sync-seed.sh`. All of these were replicated locally (coverage/llvm-cov, Docker, Pages deploy, and the LLM-CLI matrix workflows were not — see NOT-RUN list).

### 2. `cargo build --workspace` — PASS
Finished in 1m28s; both binaries produced (`target/debug/formal-ai`, `target/debug/with-formal-ai`).

### 3. Full test suite — FAIL (6 of 3358 tests, all macOS-portability; one is a real script bug)
Command: `cargo test --workspace --no-fail-fast` (first run without `--no-fail-fast` stopped after the integration target; full log: `scratchpad/audit/cargo-test-full.log`). 25 test binaries + doc-tests: **3352 passed, 6 failed, 4 ignored**. Doc-tests: 0 (none defined), pass.

Failures with output:

**(a) `unit`: `ci_cd::macos_package_retry::*` — 3 of 4 fail, flaky set varies per run**
```
thread '...' panicked at tests/unit/ci-cd/macos_package_retry.rs:106:10:
mock attempt counter must exist: Os { code: 2, kind: NotFound, ... }
```
Root cause (reproduced outside the test harness): `desktop/scripts/package-macos-with-retry.sh` line 18 uses
`mktemp ".../formal-ai-macos-package.XXXXXX.log"`. **BSD mktemp does not substitute `XXXXXX` when a suffix follows it** — it creates the literal file `formal-ai-macos-package.XXXXXX.log`, and every concurrent invocation then dies with `mktemp: mkstemp failed ... File exists`, so with `set -e` the wrapper exits before ever invoking `npx`. Four wrapper instances run concurrently under the parallel test runner → 3 of 4 fail. `cargo test --test unit macos_package_retry -- --test-threads=1` passes 4/4. Fails identically with the sandbox disabled. This is a genuine portability bug in a **macOS-targeted release script** (it runs on macOS GitHub runners in desktop-release.yml; it survives there only because a single sequential run can create the literal file once and the EXIT trap removes it — the intended tempfile randomness is absent on macOS).

**(b) `integration`: `issue_819_tui_isolation::temporary_server_diagnostics_do_not_leak_into_the_wrapped_tui` and `with_formal_ai::with_formal_ai_default_interactive_mode_launches_every_tool_in_a_pty`**
```
script: illegal option -- f
usage: script [-aeFkpqr] [-t time] [file [command ...]]
```
Test-only issue: both tests spawn `Command::new("script").args(["-qfec", ...])` (util-linux syntax) to fake a PTY; BSD `script` has no `-f`/`-e`/`-c` combo. Product code never spawns `script` (verified by grep over `src/`). Always fails on macOS, passes on the ubuntu CI runners.

**(c) `issue_757_session_files::reports_every_requested_tool_session_and_proxy_log`**
```
panicked at tests/issue_757_session_files.rs:166:9:
assertion failed: stderr.contains(&format!("  server log: {}", proxy_log.display()))
```
Root cause: `src/client_integrations.rs:486-489` canonicalizes `FORMAL_AI_PROXY_LOG` (`fs::canonicalize`). The test puts the proxy log under `std::env::temp_dir()` = `/var/folders/...`, which canonicalizes to `/private/var/folders/...` on macOS, so the printed line never matches the raw expected path. Manual repro with an already-canonical path prints the `server log:` line correctly for all 6 tool legs. Test bug on macOS; passes on Linux.

**Docs-pin subset** — PASS: `cargo test --test unit docs_requirements` → 88 passed, 0 failed.

### 4. Lints — PASS
- `cargo fmt --check` → exit 0.
- `cargo clippy --lib --bins --tests --all-features -- -D warnings` → exit 0, 0 warnings (honors `clippy.toml` doc-valid-idents; CI's exact flags).
- `cargo check --examples --all-features` → exit 0.
- `cargo doc --no-deps --lib` → exit 0, 0 warnings.
- `cargo metadata --locked` → OK (lockfile consistent).

### 5. Web bundle — PASS (with drift note)
`bun install --frozen-lockfile` (254 packages) and `bun run build:web` both succeeded; all four bundles built (vendor 0.28 MB, web-search-component 21.6 KB, ocr 18.5 KB, app.js 0.65 MB). Note: the rebuilt bundles **differ from the committed ones** (62 changed lines across the 4 files — minifier/bundler-version drift with bun 1.2.20; sample in `audit/web-bundle-drift.txt`). Restored with `git checkout` per the no-modification rule. CI rebuilds bundles at deploy time, so this is informational.

### 6. CLI hand checks (deterministic, no LLM) — PASS with 2 findings
All documented README Quick Start examples, run via the built binary (and one via `cargo run --bin formal-ai --` to confirm equivalence):
- `chat --prompt "Hi"` → `Hi, how may I help you?` + thinking trace (matches docs)
- `chat --prompt "What is 8% of $50?"` → `8% of $50 = 4 USD`
- `chat --prompt "Посчитай 1000 рублей в долларах"` → `1000 рублей в долларах = 11.1731843575419 USD`
- `chat --prompt "Write me hello world program in Rust" --format chat` → OpenAI-shaped JSON with the program, execution status ("compiled and ran in issue-8 local verification harness"), and `thinking_steps` — matches docs
- `formal-ai agent --help` → OK. Tiny offline run `agent --silent --task "Formalize «The cat sat on the mat»..."` → exit 0, **but the output is the default fairy-tale knowledge base** (`tale:fisherman-and-fish`, header "Formalized «Сказка о рыбаке и рыбке»") — the custom `--task` text is silently not reflected; flagged as a finding (driver routes to its seeded formalize capability with no warning).
- Server: `formal-ai serve --host 127.0.0.1 --port 18080` started; `POST /api/openai/v1/chat/completions` returned a valid chat completion with `thinking_steps`; server stopped cleanly.
- `with-formal-ai --help` → OK.

### 7. Determinism — PASS
`chat --prompt "What is 8% of $50?"` run twice → `cmp` byte-identical.

### 8. Multilingual — PARTIAL FAIL (hi/zh word arithmetic)
- Greetings: `Hello` / `Привет` / `नमस्ते` / `你好` → correct localized greeting in each language. PASS.
- Word-operator arithmetic: `What is 2 plus 2?` → `2 plus 2 = 4`; `Сколько будет 2 плюс 2?` → `2 плюс 2 = 4`; but Hindi `2 जमा 2 कितना होता है?` / `2 जोड़ 2 कितना होता है?` and Chinese `2 加 2 等于多少?` both fall to the **unknown handler** ("could not determine ... Report issue"). Symbolic `2 + 2` works in all languages. This contradicts the README/USER-JOURNEYS claim that "every operation is recognized equally across en | ru | hi | zh"; the seeded operation vocabulary has `相加` (zh) but the `加`/`जोड़`/`जमा` infix forms are not recognized.

### 9. Desktop + VS Code smokes — PASS (both)
- `npm --prefix desktop install && npm --prefix desktop run smoke` → "formal-ai desktop smoke checks passed"
- `npm --prefix vscode install && npm --prefix vscode run smoke` → "formal-ai vscode smoke checks passed"
- Note: puppeteer's Chrome download failed during both installs (network-restricted environment); the smoke scripts don't need it. Puppeteer-based e2e tests were therefore NOT run (env-blocked).
- Bonus CI gates: `npm ci --prefix tests/e2e` + all 6 static checks (`check:i18n`, `check:language-parity`, `check:language-test-coverage`, `check:intent-coverage`, `check:web-tdz`, `check:web-hardcoded-ui`) → all PASS.

### 10. scripts/sync-seed.sh sanity — EXPECTED-DIVERGED + 1 script bug
- `src/web/seed/` does not exist locally and is **gitignored** (deploy mirror generated by the script/CI before Pages upload), so `diff -r data/seed src/web/seed` reporting everything "Only in data/seed" is the expected local state, not breakage. The copy mode was NOT run (it writes into `src/`).
- Real finding: `scripts/sync-seed.sh --check` crashes on stock macOS bash 3.2 at line 62: `dests[@]: unbound variable` (empty-array expansion under `set -u`, fixed in bash 4.4+). The check still exits 1 correctly here, but the orphan-detection pass never runs on macOS.

## NOT-RUN (with reasons)
- Puppeteer-based desktop/vscode e2e tests — env-blocked (Chrome download refused by network policy).
- `cargo llvm-cov` coverage job — tool-heavy duplicate of the test run; tests already executed in full.
- Docker runtime checks (`verify-docker-runtime.sh`), Pages deploy, crates.io publish steps — deploy-only, need Docker/credentials.
- `agentic-cli-matrix` / `external-benchmarks` / `learning-cycle` workflows — require network LLM CLIs and API keys.
- `cargo test --workspace` on Linux — not available here; the 6 macOS failures are asserted to pass on ubuntu CI only by root-cause analysis, not by execution.

## Broken things (ranked)

1. **HIGH — `desktop/scripts/package-macos-with-retry.sh` mktemp template is not BSD-portable** (`formal-ai-macos-package.XXXXXX.log`): macOS mktemp creates the literal file, concurrent runs fail with `File exists`, wrapper aborts before `npx`; causes 3 flaky `ci_cd::macos_package_retry` unit-test failures locally and removes the intended tempfile randomness on the very OS the script targets in desktop-release.yml.
2. **MEDIUM — multilingual parity gap**: Hindi/Chinese word-operator arithmetic (`2 जोड़ 2`, `2 加 2`) routes to the unknown handler while the same question succeeds in en/ru, contradicting the documented equal-language promise.
3. **MEDIUM — macOS test bug**: `tests/issue_757_session_files.rs:166` compares against the un-canonicalized proxy-log path; product canonicalizes `/var` → `/private/var` (src/client_integrations.rs:486-489), so the test always fails on macOS.
4. **MEDIUM — macOS test bug**: `tests/integration/issue_819_tui_isolation.rs` and `tests/integration/with_formal_ai.rs` fake a PTY with util-linux `script -qfec`; BSD `script` has no such options, so 2 integration tests always fail on macOS (product code unaffected).
5. **LOW — `scripts/sync-seed.sh --check`** dies with `dests[@]: unbound variable` on macOS bash 3.2 when the destination mirror is empty.
6. **LOW/INFO — committed web bundles drift** from what bun 1.2.20 produces (`bun run build:web` yields a different minified output than the checked-in files); harmless for releases (CI regenerates) but `git status` gets dirty after any local web build.
7. **LOW/INFO — `formal-ai agent --task <custom>` silently ignores the custom task** and emits the default fairy-tale knowledge base with exit 0.

## Key artifacts
- Full test log: `scratchpad/audit/cargo-test-full.log` (first, fail-fast run: `cargo-test.log`)
- Clippy log: `scratchpad/audit/clippy.log`; cargo doc log: `cargodoc.log`
- Web bundle drift sample: `scratchpad/audit/web-bundle-drift.txt`
- Seed diff: `scratchpad/audit/seed-diff.txt`; determinism pair: `det1.txt`/`det2.txt`
