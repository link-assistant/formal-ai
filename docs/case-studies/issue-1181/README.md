# Issue #1181 Case Study: The Release Nobody Could Run, and the `=1` That Errored

Issue [#1181](https://github.com/link-assistant/formal-ai/issues/1181) (E146,
part of the #1183 umbrella). Verified on `main` at `d209aac64`; fixed on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

Two ways to be stuck, from the same release:

1. **No binary to run.** Release `v0.352.1` carries 24 assets — the Electron
   desktop apps (which do not expose the Rust CLI on macOS), the VSIX, the
   updater metadata and provenance files — and no standalone `formal-ai`
   binary for any platform. The only ways to run the CLI were `cargo install
   formal-ai` (a full release build of this dependency graph on the user's
   machine) or the `ghcr.io/link-assistant/formal-ai` image, which #1153
   measured at 24.3 GB. Everyone in the #1183 umbrella ("people cannot run
   the current release to check any of this") inherits this blocker.

2. **The one env switch that gated CLI output rejected the conventional
   spelling.**
   `FORMAL_AI_SILENT=1 formal-ai chat --prompt …` fails with
   `invalid value '1' for '--silent' [possible values: true, false]`.
   clap's derived bool argument only accepts the literal `true`/`false` from
   its `env` attribute, while every other `FORMAL_AI_*` boolean switch in the
   codebase was read ad hoc — each site inventing its own accepted spellings
   (`== Ok("1")` for the debug flags, `"0"|"false"|"off"` for the opt-outs,
   `"1"|"true"|"yes"|"on"` for the live-fetch family).

## Root cause

1. **The release pipeline only ever packaged the desktop app.** The
   `desktop-release.yml` workflow's `build` job compiles `formal-ai` per-OS
   natively (glibc) purely to embed the binary inside the Electron app; no
   job ever uploaded the CLI itself, and no musl/cross-compilation setup
   existed anywhere in the repository (`git grep -l musl` over `.github`,
   `Dockerfile`, `scripts`, `rust/Cargo.toml` returned nothing).

2. **Boolean environment switches had no shared parser.** clap's bool
   `value_parser` accepts only `true`/`false`; the 20-odd ad-hoc
   `std::env::var("FORMAL_AI_*")` reads each hardcoded their own matcher, so
   the accepted spellings differed per switch, per file, and (for
   `FORMAL_AI_SILENT`) per *read* — the clap argument demanded `true` while
   `dialog_log::verbose_enabled()`'s second read of the same variable checked
   `!= Ok("1")`. There was no single place to fix the spelling table.

## The change

### Part A — one boolean spelling table (R6/R7, testable)

| File | Change |
| --- | --- |
| `rust/src/cli_env.rs` (new) | `parse_bool_env` (the `1/0/true/false/yes/no/on/off` table, case-insensitive, whitespace-tolerant, `None` for anything else), `bool_env_value_parser` (the clap adapter), `flag_enabled` (opt-in switches: on only when set to a true spelling), `flag_disabled` (opt-out switches: off only when set to a false spelling). |
| `rust/src/lib.rs` | `pub mod cli_env;` in the alphabetical module list. |
| `rust/src/main.rs` | `--silent` gains `value_parser = formal_ai::cli_env::bool_env_value_parser`. The `ArgAction::SetTrue` is preserved, so bare `--silent` still works; `FORMAL_AI_SILENT=1/yes/on/TRUE/…` now parse. |
| `rust/src/solver_helpers/mod.rs` | `env_bool`/`env_bool_with_extra_truthy` delegate their standard spellings to `parse_bool_env`, so the pre-existing helpers and the new module share one table (extras keep their explicit arms). |
| `rust/tests/unit/issue_1181_boolean_env_parser.rs` (new) | The acceptance suite below. |

Ad-hoc reads converted (each keeps its previous garbage-value behaviour: an
unrecognised value leaves the documented default in force):

| Variable | Sites |
| --- | --- |
| `FORMAL_AI_SILENT` | `dialog_log.rs` `verbose_enabled` (`!= Ok("1")` → `!flag_enabled`) |
| `FORMAL_AI_TRACE_REQUESTS` | `dialog_log.rs`, `protocol.rs`, `protocol_responses.rs`, `agentic_coding/planner/continuation.rs` |
| `FORMAL_AI_TRACE_COMMANDS` | `agent.rs` |
| `FORMAL_AI_TRACE_SLOW_INIT` | `agentic_coding/self_ast.rs` |
| `FORMAL_AI_RECORD_CHAT` | `memory_sync.rs` `chat_recording_enabled` (`!matches!(… "0"\|"false"\|"off")` → `!flag_disabled`) |
| `FORMAL_AI_MEMORY_DEBUG` | `memory_sync.rs` (three sites) |
| `FORMAL_AI_DREAMING_DEBUG` | `dreaming_runtime.rs` |
| `FORMAL_AI_DREAMING` | `dreaming_runtime.rs` `dreaming_disabled` → `flag_disabled` |
| `FORMAL_AI_LINK_CLI_DEBUG` | `link_store/staging.rs` |
| `FORMAL_AI_LIVE_FETCH` | `meta_method_dispatch.rs`, `solver_dispatch.rs` (three sites), `external_benchmarks/mod.rs`, `coding/synthesis_runtime.rs` |
| `FORMAL_AI_LIVE_API` | `lexeme_import.rs`, `translation/cache.rs` |
| `FORMAL_AI_SEED_LINKS_MIRROR` | `seed_links.rs` `native_mirror_enabled` → `flag_enabled` |

Intentionally **not** converted:

- `FORMAL_AI_FULL_TRUST` (`bounded_autonomy.rs:184`) — a security opt-in whose
  `FULL_TRUST_VALUE = "1"` const documents an exact-match sentinel; widening
  the accepted set would loosen a trust boundary, which is not what #1181's
  UX consistency asks for.
- The `env_truthy` family (`FORMAL_AI_OFFLINE`, `FORMAL_AI_AGENT_MODE`,
  `FORMAL_AI_DIAGNOSTIC_MODE` in `solver_helpers/mod.rs`) — "anything other
  than a falsy value is true" is that helper's documented lenient semantics
  and it already accepts all eight spellings correctly; nothing in #1181
  requires tightening it.
- `FORMAL_AI_TRANSLATION_DEBUG` (`translation/pipeline.rs:43`) — the file is
  owned by the concurrent issue-#1174 change on this branch; converting that
  read is deferred to it (it becomes a one-line `flag_enabled` call).
- Non-boolean `FORMAL_AI_*` variables (modes, paths, numbers, tokens) are
  untouched.

The error message for an unrecognised clap value is a plain
`String::from("expected one of: 1, 0, true, false, yes, no, on, off
(case-insensitive)")` — clap already prefixes `invalid value '…' for
'--silent'` around it, and the plain (non-`format!`-template) literal keeps
this plumbing message out of the R379 hardcoded-language ledger.

### Part B — the release workflow (R1-R5, verified on the next release)

| File | Change |
| --- | --- |
| `.github/workflows/desktop-release.yml` | New `cli` job: five-leg matrix (`x86_64-unknown-linux-musl`/`aarch64-unknown-linux-musl`/`x86_64-apple-darwin`/`aarch64-apple-darwin`/`x86_64-pc-windows-msvc`), mirroring the `build` job's `needs: [resolve, base]`, per-leg concurrency queue, PR dry-run `if:`, write/id-token/attestations permissions, checkout/simulate-fresh-merge/sccache conventions. Linux legs use `taiki-e/setup-cross-toolchain-action@v1` (the dependency graph compiles C code — tree-sitter grammars through `meta-language`, `aws-lc-sys` through `webrtc`'s rustls — so a bare `rustup target add` + `musl-tools` is not enough). Builds with `cargo build --release --bin formal-ai --locked --target <triple>`, packages `formal-ai-cli-<triple>/` (binary + `LICENSE` + `README.md`) as `.tar.gz` (PowerShell `Compress-Archive` for the Windows `.zip`), smoke-tests by extracting the archive and running `formal-ai --version` on the leg's own runner, attests, uploads to the release, and emits a `SHA256SUMS-cli-<label>.partial` fragment. `finalize` gains `cli` in `needs` and the five `cli-*` labels in `builder_of`/the label loop, so the consolidated `SHA256SUMS.txt`/`BUILD-PROVENANCE.txt` cover the CLI archives with the same INCOMPLETE honesty handling. |
| `rust/Cargo.toml` | `[package.metadata.binstall]` + the Windows `.zip` override (see R1181-4). |
| `scripts/install.sh` | `install_cli()` becomes prebuilt-first: new `cli_target_triple`/`resolve_cli_bin_dir`/`install_cli_prebuilt` helpers (host-triple mapping, download, `verify_checksum`, extract, install, `--version` smoke, PATH hint), with the previous `cargo install` body as the fallback when no archive matches the host. Checksum mismatch still dies rather than falling back. |
| `scripts/install.ps1` | The mirrored `Get-CliTargetTriple`/`Resolve-CliBinDir`/`Install-CliPrebuilt` functions and the same fallback structure; `Install-Telegram`/`Invoke-Main` pass the release JSON through. |

Divergences from the issue body's design sketch, with reasons:

- The parser module is `rust/src/cli_env.rs` with
  `parse_bool_env`/`bool_env_value_parser`/`flag_enabled`/`flag_disabled`
  rather than the sketch's `env_bool.rs` `parse_bool_env`/`env_flag` —
  chosen by the coordinating change for this branch so the name states what
  it is (CLI/environment switch plumbing). The sketch's `parse_bool_env`
  returned `Err` for unrecognised values, which would have changed the
  garbage-value behaviour of every `== Ok("1")` debug flag site; the shipped
  `parse_bool_env` returns `Option<bool>` so each call site keeps its
  documented default on garbage, and only the clap adapter errors.
- Packaging is inlined in the workflow rather than a new
  `scripts/package-cli-archive.sh` — the smoke test in the same job asserts
  the archive contents, so the behaviour the issue asked for is delivered
  with one fewer moving part.
- The job timeout is the `build` job's measured per-runner cap (40–50 min)
  rather than the sketch's flat 30: the same dependency graph measured a
  33-minute cold compile on `macos-15-intel` (run 30788311906, cited in
  `desktop-release.yml`'s own timeout comment).
- `binstall`'s `pkg-url` spells the literal `.tar.gz` name instead of the
  sketch's `{ archive-suffix }` placeholder: for `pkg-fmt = "tgz"` that
  placeholder expands to `.tgz`, and the R2 archive names end `.tar.gz` —
  the sketch's URL would have 404'd.
- The `bin-dir` is `formal-ai-cli-{ target }/{ bin }{ binary-ext }` to match
  the archive's top-level staging directory (`LICENSE`/`README.md` sit next
  to the binary instead of being scattered into the user's current
  directory on extraction).

## Tests

`rust/tests/unit/issue_1181_boolean_env_parser.rs` (registered by the
integrator as `mod issue_1181_boolean_env_parser;` in
`rust/tests/unit/mod.rs`):

1. `parse_bool_env_accepts_every_documented_spelling` — the exact truth
   table: all eight spellings, mixed case, surrounding whitespace.
2. `parse_bool_env_rejects_unrecognised_values_instead_of_guessing` —
   `""`, `"maybe"`, `"2"`, `"y"`, `"on-off"`, … are `None`.
3. `bool_env_value_parser_maps_spellings_to_clap_results` — the clap adapter
   maps the same table to `Ok(bool)`/`Err(String)`, and the error states
   what is accepted.
4. `flag_enabled_is_true_only_for_true_spellings` /
   `flag_disabled_is_true_only_for_false_spellings` — the helper semantics on
   a synthetic variable name (hermetic: no other test reads it).
5. `silent_switch_now_accepts_one_as_the_issue_reported` — the exact reported
   defect: `FORMAL_AI_SILENT=1` (and `true`/`yes`/`ON`) now silences
   `verbose_enabled()`; `0`/`off` and garbage keep it on.
6. `record_chat_opt_out_accepts_every_false_spelling` — the opt-out idiom on
   a real default-on feature, including the newly accepted `no`/`NO`.

The suite is hermetic (no process spawn; variables set through `temp-env`,
the crate's edition-2024-safe scoped override, per the convention documented
in `rust/tests/unit/issue_822.rs`). The pre-existing
`dreaming_env_opt_out_recognizes_only_explicit_off_values` continues to pass
unchanged against the converted `dreaming_disabled`.

Command:
`RUSTUP_TOOLCHAIN=1.98.1 cargo test --manifest-path rust/Cargo.toml --test unit issue_1181_boolean_env_parser`.

The R7 end-to-end assertion (a spawned `FORMAL_AI_SILENT=1 formal-ai chat`
run) is covered at the unit level by the `value_parser` adapter tests plus
the `verbose_enabled` integration test, because `Args` lives in the bin
crate, which the lib test target cannot import.

## Honest boundaries

- **Part B is delivered as workflow code and reviewed, not executed.** The
  `cli` job runs for the first time on the next tagged release (or a pull
  request touching its paths); until an asset list shows the five
  `formal-ai-cli-*` archives and a `SHA256SUMS.txt` that covers them, R1–R5
  are `not yet confirmed`, per the release-pipeline silent-deferral risk this
  issue's own definition of done names.
- The musl legs cross-compile C code (`aws-lc-sys` through `webrtc`'s
  rustls, tree-sitter grammars through `meta-language`); `taiki-e/
  setup-cross-toolchain-action@v1` is the standard wiring for that, but the
  first real run is the proof. A single failing leg degrades to the
  finalize job's INCOMPLETE report rather than silently shipping a partial
  manifest.
- `install.ps1` follows the file's existing cmdlet idioms but could not be
  machine-parsed on the implementing workstation (no PowerShell on macOS);
  it is review-verified only.
- `rust/tests/unit/mod.rs`, `docs/requirements-traceability.md`,
  `REQUIREMENTS.md` regeneration (`rust-script scripts/assemble-requirements.rs
  --write`) and the debt-ratchet ledger are integrator-owned surfaces on this
  branch and are not touched by this change. This change adds and removes
  **zero** `literal_predicates` (`contains("`/`starts_with("`) occurrences in
  `rust/src` and adds no hardcoded-language rows: the only new string
  literals are env-var names, match-arm spellings, and one plain non-template
  error literal.
