# Regression and verification command index

## Preserved red reproductions

### Dangling dependency

Command:

```text
cargo test --test unit issue_845_fact_checking -- --test-threads=1
```

Pre-fix failure:

```text
assertion failed: a reference to absent evidence must not be reported as admitted evidence
  left: EvidenceWeighted
 right: PriorOnly
```

Local full log:
`dev/log/issues/845/pulls/856/red-dangling-dependency.log`.

### Missing live Rust routing

Command:

```text
cargo test --test integration issue_845_fact_checking -- --test-threads=1
```

Pre-fix behavior: the en/ru/hi/zh current-dialogue requests did not reach a
`fact_check_current_dialogue` handler because no such runtime handler existed.

Local full log: `dev/log/issues/845/pulls/856/red-runtime-routing.log`.

### Missing browser-worker parity

Command:

```text
npx playwright test --config=playwright.local.config.js tests/issue-845.spec.js
```

Pre-fix behavior: English was routed to live web search
(`Search results for fact-check this dialogue`); Russian, Hindi, and Chinese
were routed to unknown-answer fallbacks.

Local full log:
`dev/log/issues/845/pulls/856/red-browser-worker-parity.log`.

## Focused green verification

The continued session runs and preserves these local logs:

```text
cargo test --all-features --test unit issue_845_fact_checking -- --test-threads=1
cargo test --all-features --test integration issue_845_fact_checking -- --test-threads=1
npx playwright test --config=playwright.local.config.js tests/issue-845.spec.js
python3 scripts/generate-role-registry.py
python3 scripts/close-total.py
python3 scripts/audit-total-closure.py
scripts/sync-seed.sh --check
cargo run --example regenerate_self_ast_census
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib
DOCS_RS=1 RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib --no-default-features
rust-script scripts/check-file-size.rs
rust-script scripts/check-hardcoded-language.rs
cargo test --all-features --verbose -- --test-threads=2
cargo test --all-features --test unit --verbose
cargo test --doc --verbose
npx playwright test --config=playwright.local.config.js --shard=1/2
npx playwright test --config=playwright.local.config.js --shard=2/2
npx playwright test --config=playwright.local.config.js tests/issue-336.spec.js \
  --grep "agent mode works across supported UI languages"
npx playwright test --config=playwright.local.config.js tests/issue-501.spec.js \
  --grep "routes install requests"
```

Final outcomes:

- issue #845 unit suite: 12 passed;
- issue #845 integration suite: 3 passed;
- complete unit suite: 2,119 passed, 2 ignored;
- complete integration suite: 241 passed;
- documentation tests: 0 failed;
- complete browser matrix: 436 passed, 1 intentional skip, with no issue #845
  failure.

The unsharded browser command reached 333 passing cases before the repository's
900-second global timeout stopped the run. The two shards covered all 437
configured cases. Issue #336 timed out at 30 seconds in shard 1 and passed
unchanged in isolation in 29.8 seconds. Issue #501 timed out at 30 seconds in
shard 2 and passed unchanged in isolation in 22.7 seconds. These load-sensitive
results are preserved in:

```text
dev/log/issues/845/pulls/856/full-playwright.log
dev/log/issues/845/pulls/856/full-playwright-shard-1.log
dev/log/issues/845/pulls/856/full-playwright-shard-2.log
dev/log/issues/845/pulls/856/playwright-issue-336-isolated.log
dev/log/issues/845/pulls/856/playwright-issue-501-isolated.log
```

The first full Cargo attempt used the local default test concurrency and caused
15 unrelated HTTP integration tests to hit the same 30-second `WouldBlock`
response deadline. The exact failing route passed alone, and the entire
241-test integration suite passed with `--test-threads=2`. A subsequent full
unit run passed all 2,119 tests after updating the expected contextual-handler
registry count from seven to eight. The preserved logs are:

```text
dev/log/issues/845/pulls/856/full-cargo-test-default-concurrency.log
dev/log/issues/845/pulls/856/full-cargo-test-stale-registry-count.log
dev/log/issues/845/pulls/856/full-unit-test.log
dev/log/issues/845/pulls/856/doctest.log
```
