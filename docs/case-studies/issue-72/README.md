# Issue 72 Case Study: GitHub Pages Demo Advertises Stale formal-ai Version

## Summary

Issue [#72](https://github.com/link-assistant/formal-ai/issues/72) reported that the GitHub Pages demo
at <https://link-assistant.github.io/formal-ai/> still advertised version `0.16.0` even though the
crate had moved to `0.38.0`. The original screenshot shows a deployment from PR #36 that was Active
when the issue was filed.

The original `src/web/index.html` pinned the version with a hardcoded `<meta>` literal that the
release pipeline never updated. The fix replaces the literal with a `__FORMAL_AI_VERSION__`
placeholder, teaches `scripts/stamp-pages-artifact.sh` to substitute it during the GitHub Pages
deploy from the live `Cargo.toml`, and surfaces the same `CARGO_PKG_VERSION` value through the
CLI's `--version` flag and a new Telegram `/version` command. Tests in `tests/unit/ci-cd/` and
`tests/unit/mvp/` reject the placeholder/literal regression structurally and through a smoke
subprocess that runs the stamp script end to end.

## Collected Data

Raw GitHub data is preserved under this directory:

- `raw-data/issue-72.json`, `raw-data/issue-72-comments.json`: issue body and follow-up comments.
- `raw-data/pr-77.json`, `raw-data/pr-77-conversation-comments.json`,
  `raw-data/pr-77-review-comments.json`, `raw-data/pr-77-reviews.json`: prepared PR data.
- `screenshot-original.png`: the deployment screenshot from the issue showing the active PR #36
  page that advertised the stale 0.16.0 version.
- `template-data/{rust,js,python,csharp}-template-release.yml`: snapshots of every
  AI-driven-development-pipeline template's `release.yml` used in the template comparison below.

## Requirements

From issue #72 and the follow-up comment from `@konard` the work must:

1. Display the live release in the web app, in the Telegram bot's `/version` command, and in the
   CLI's `--version` flag.
2. Keep CLI/bot argument parsing consistent with `link-foundation/lino-arguments` (drop-in
   clap-style API). The repository already uses `lino-arguments = "0.3"`, so the fix must continue
   to use it.
3. Double-check CI/CD for false positives so the version stamped in every surface is the
   version actually published.
4. Compare with the four pipeline templates and report a follow-up issue in any template that
   shares the same bug.
5. Collect issue data into `docs/case-studies/issue-72/` and do a deep case study analysis,
   including online research and a list of every requirement plus its solution plan.
6. Plan and execute everything in a single pull request (PR
   [#77](https://github.com/link-assistant/formal-ai/pull/77)).

## Root Cause

The release pipeline bumped `Cargo.toml` and `Cargo.lock`
(`scripts/version-and-commit.rs`) but never touched `src/web/index.html`. The web demo therefore
served whatever version was last hand-edited into `<meta name="formal-ai-version" content="..."/>`.
Pre-fix, that literal was `0.16.0` and had been since well before v0.38.0.

The `deploy-demo` job uploaded `src/web/` verbatim to GitHub Pages via `actions/upload-pages-artifact@v5`
and `actions/deploy-pages@v5`. A stamp step (`scripts/stamp-pages-artifact.sh`) already substituted
`__FORMAL_AI_ASSET_VERSION__` placeholders to bust browser caches, but it ignored the version
meta tag because that field used a real literal rather than a placeholder. There was no test
that diffed the deployed meta tag against `Cargo.toml`, so the drift went undetected.

`src/web/app.js` mirrored the same literal as a JS-level fallback:

```js
const APP_VERSION =
  document.querySelector('meta[name="formal-ai-version"]')?.content || "0.16.0";
```

Even if the meta tag had been corrected, the fallback would have continued to advertise `0.16.0`
when run in a context where the meta tag was missing (for example a local Playwright run
serving `src/web/` directly).

The CLI and Telegram bot did not surface the version at all:

- `src/main.rs` declared the parser with `#[command(name = "formal-ai", about = "...")]`. Without
  the `version` attribute clap does not wire up `-V/--version`.
- `src/telegram.rs` routed every message through `reply_for_message` which always called the
  symbolic engine. No `/version` bot command existed.

## Solution

### Web app

1. `src/web/index.html` now declares `<meta name="formal-ai-version" content="__FORMAL_AI_VERSION__"/>`.
   The placeholder follows the same convention as the existing
   `__FORMAL_AI_ASSET_VERSION__` cache-buster.
2. `scripts/stamp-pages-artifact.sh` accepts a 4th positional argument (or `FORMAL_AI_VERSION`
   env var) and substitutes the placeholder during the Pages deploy. When the caller omits the
   argument it reads `Cargo.toml` directly via `sed` so the script remains usable from anywhere
   that has the repository checked out, without needing `rust-script` in the deploy-demo job.
3. The stamp step in `.github/workflows/release.yml` reads the live `Cargo.toml` version in a
   prior `Read formal-ai version from Cargo.toml` step (also using inline `sed`, because the
   deploy-demo job does not set up Rust) and forwards it as the 4th positional argument.
4. `src/web/app.js` now treats unreplaced placeholders as a "dev" build instead of pinning
   `0.16.0`. This keeps local Playwright runs sane while making it impossible for the production
   meta tag to advertise a stale literal.
5. `scripts/wait-for-pages-deployment.sh` rejects both `__FORMAL_AI_ASSET_VERSION__` and
   `__FORMAL_AI_VERSION__` placeholders in the served index so a half-stamped deploy fails
   the live e2e wait instead of silently going green.
6. `scripts/stamp-pages-artifact.sh` writes the rendered crate version into the
   `deployment.json` marker (alongside the asset version and SHA) so post-deploy probes can
   compare published and configured versions.

### CLI

`src/main.rs` now uses `#[command(name = "formal-ai", version, about = "...")]`. The `version`
attribute makes clap (and therefore `lino-arguments`, which re-exports clap's `Parser`
derive) read `CARGO_PKG_VERSION` automatically, so `formal-ai --version` prints
`formal-ai 0.38.0` without any manual bookkeeping.

### Telegram bot

`src/telegram.rs` adds a tiny pre-check in `reply_for_message` that recognizes `/version` and
`/version@bot-name` regardless of case and replies with `formal-ai <CARGO_PKG_VERSION>`. The
reply is wrapped in the same HTML/`parse_mode=HTML` envelope every Telegram reply uses, so the
existing test infrastructure exercises it end to end.

### Why `CARGO_PKG_VERSION`

Every surface now sources its number from one place: cargo. clap exposes it via the `version`
attribute, the Telegram bot uses the `env!("CARGO_PKG_VERSION")` macro, and the GitHub Pages
deploy reads `Cargo.toml` with `sed`. Because `cargo publish` and `scripts/version-and-commit.rs`
both update `Cargo.toml`, every release path remains the single source of truth. Tests assert
that the deploy workflow forwards that value to the stamp script and that none of the surfaces
fall back to a hardcoded literal.

## Regression Coverage

| Test | Layer | What it asserts |
| --- | --- | --- |
| `tests/integration/formal_ai_cli.rs::cli_version_flag_prints_crate_version` | CLI | `formal-ai --version` prints `formal-ai <CARGO_PKG_VERSION>` exactly. |
| `tests/unit/mvp/telegram_surface.rs::telegram_version_command_replies_with_crate_version` | Telegram | `/version` in a private chat replies with `formal-ai <CARGO_PKG_VERSION>` and no `/trace` suffix. |
| `tests/unit/mvp/telegram_surface.rs::telegram_version_command_with_bot_suffix_still_replies` | Telegram | `/version@formal_ai_bot` in a group chat is recognized regardless of the bot-name suffix. |
| `tests/unit/ci-cd/workflow_release.rs::github_pages_artifact_advertises_crate_version_from_cargo_toml` | CI/CD wiring | `index.html` carries the placeholder, `app.js` carries no `"0.16.0"` literal, the stamp script substitutes it and validates the rendered meta tag, the deploy workflow reads the crate version and forwards it to the stamp script, and the wait script rejects both placeholders. |
| `tests/unit/ci-cd/workflow_release.rs::stamp_pages_artifact_replaces_formal_ai_version_placeholder` | Stamp script smoke | Runs `scripts/stamp-pages-artifact.sh` against a scratch copy of `src/web/index.html` and asserts the rendered meta tag advertises the supplied version and `deployment.json` records it. |

The existing
`tests/unit/ci-cd/workflow_release.rs::static_demo_runtime_assets_are_cache_busted_by_deployment_version`
test already pinned the `__FORMAL_AI_ASSET_VERSION__` half of the contract; the two new tests
extend it to cover `__FORMAL_AI_VERSION__`.

## Template Comparison

The follow-up comment asked the fix to be cross-checked against the four
`link-foundation/{rust,js,python,csharp}-ai-driven-development-pipeline-template` repositories. I
captured each template's `release.yml` under `template-data/` for the historical record.

- `rust-ai-driven-development-pipeline-template`: deploys `cargo doc` output (`target/doc`) to
  GitHub Pages. The HTML is generated by cargo and already reads the live crate version, so the
  same "stale literal in static HTML" bug cannot occur upstream. No template issue is needed.
- `js-ai-driven-development-pipeline-template`: no GitHub Pages deployment step.
- `python-ai-driven-development-pipeline-template`: no GitHub Pages deployment step.
- `csharp-ai-driven-development-pipeline-template`: no GitHub Pages deployment step.

The bug pattern (hardcoded version literal inside a hand-written static site that the release
pipeline does not rewrite) only manifests in this repository because `formal-ai` deploys its own
custom React demo, not a cargo-generated artifact. The relevant generalizable lesson — never
ship a version literal in static deployment artifacts; always either generate it from the build
metadata or stamp it from a placeholder during deploy — is preserved here so future repositories
that adopt a similar custom-demo pattern can copy the placeholder + stamp script approach.

## Online Research

- [clap derive `version` attribute](https://docs.rs/clap/latest/clap/_derive/index.html#command-attributes):
  `#[command(version)]` tells clap to use `CARGO_PKG_VERSION` from the package metadata.
- [`env!` macro](https://doc.rust-lang.org/std/macro.env.html): used in the Telegram handler to
  read `CARGO_PKG_VERSION` at compile time without adding any runtime configuration surface.
- [actions/deploy-pages](https://github.com/actions/deploy-pages) and
  [actions/upload-pages-artifact](https://github.com/actions/upload-pages-artifact): the
  artifact upload step ships the entire `src/web/` directory as-is, which is why the stamp must
  run before upload rather than after.
- [Telegram Bot API bot commands](https://core.telegram.org/bots/features#commands): bot
  commands are sent as `/<command>` in private chats and `/<command>@<bot-name>` in groups.
  Stripping the `@<bot-name>` suffix is necessary to recognize the command in both contexts.

## Verification

Local checks executed before pushing the fix (all green):

- `cargo run --quiet --bin formal-ai -- --version` → `formal-ai 0.38.0`
- `tmpdir=$(mktemp -d) && cp -r src/web "$tmpdir/" && scripts/stamp-pages-artifact.sh "$tmpdir/web" abc abc 9.9.9` then
  `grep formal-ai-version "$tmpdir/web/index.html"` → `<meta name="formal-ai-version" content="9.9.9" />`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test` (224 unit + 76 framework + 8 integration tests pass; pre-existing 69 `#[ignore]`d
  MVP-target tests remain ignored)
- `cargo test --test integration cli_version_flag_prints_crate_version`
- `cargo test --test unit telegram_version`
- `cargo test --test unit github_pages_artifact_advertises_crate_version_from_cargo_toml`
- `cargo test --test unit stamp_pages_artifact_replaces_formal_ai_version_placeholder`

The next live `main` push will exercise the production stamp flow end to end through the
`deploy-demo` and `test-e2e-pages` jobs, with `wait-for-pages-deployment.sh` rejecting any
deployment that still serves the `__FORMAL_AI_VERSION__` placeholder.
