# Issue #961 — macOS CI parity

Issue: <https://github.com/link-assistant/formal-ai/issues/961>

Pull request: <https://github.com/link-assistant/formal-ai/pull/987>

## 1. Collected data

The original macOS audit is preserved at
[`../issue-957/raw-data/test-report.md`](../issue-957/raw-data/test-report.md).
The issue snapshot and its empty comment thread are under `raw-data/github/`.
The audit was run on macOS 15.7.7 / Darwin 24.6.0 with the system Bash 3.2.57;
it recorded 3,352 passing tests, six failures, and four ignored tests. All six
failures came from the four portability defects grouped into issue #961.

## 2. Timeline

- **2026-08-03** — the full local macOS audit records the four defects in
  `test-report.md` under “Broken things” items 1, 3, 4, and 5.
- **2026-08-04** — issue #961 groups the findings into one macOS-parity change
  and requires a single pull request, direct regression coverage, and a macOS
  full-suite CI leg.
- **2026-08-10** — PR #987 starts from `main` at `ee94b66f`; its issue and PR
  threads contain no comments, reviews, or inline review comments before work.
- **2026-08-10** — a tests-first source-contract suite is added and run against
  the unfixed files. All five tests fail: one for each of the four defects and
  one whole-task composition test.
- **2026-08-10** — the first fresh macOS CI leg exposes a bootstrap blocker
  before Cargo: the shared disk cleanup script passes GNU-only
  `--output=avail` to BSD `df`. Two new tests reproduce the exit-64 failure
  before the helper is changed to its POSIX output form.
- **2026-08-10** — the next macOS leg reaches the complete integration suite
  and exposes an input-order race in the migrated interactive PTY test. A new
  tests-first contract requires the caller to wait for its fake TUI's readiness
  marker; focused unit and eight-client integration tests verify the repair.
- **2026-08-10** — a third macOS leg proves readiness alone is insufficient:
  BSD `script` prints `TUI_READY`, terminal EOT, then `hi`. A behavioral
  regression reproduces the early EOF before the helper keeps stdin open until
  the PTY command exits.

## 3. Requirements

| ID | Requirement | Regression proof |
| --- | --- | --- |
| R961-1 | Give the macOS packaging retry wrapper a BSD-portable random log path. | `package_log_uses_a_bsd_portable_mktemp_template`; the existing parallel `macos_package_retry` tests exercise collision freedom. |
| R961-2 | Compare the session diagnostic with the same canonical proxy-log path the product prints. | `proxy_log_expectation_matches_the_canonicalized_product_path` plus a real symlink-alias integration case in `issue_757_session_files`. |
| R961-3 | Run both PTY integration tests with the platform’s `script(1)` syntax. | `pty` helper unit tests pin exact BSD and util-linux argv; the interactive caller waits for a line-ending-independent readiness marker and keeps the input pipe open until the PTY command exits. |
| R961-4 | Let `sync-seed.sh --check` reach orphan detection under Bash 3.2 when the destination is empty. | Source-order contract plus `seed_sync_reaches_the_orphan_pass_with_an_empty_destination`. |
| R961-5 | Run the complete Rust test suite on macOS in CI. | `full_test_matrix_runs_on_a_supported_macos_image` pins `macos-15-intel` beside Linux; source and fake-BSD execution tests ensure the shared disk-cleanup bootstrap reaches Cargo. |
| R961-6 | Keep a whole-task regression and the required issue/PR evidence. | `complete_macos_portability_contract_holds`, this case study, and the PR #987 case study. |

## 4. Root causes

**RC1 — the random portion was not the BSD template suffix.** The wrapper used
`formal-ai-macos-package.XXXXXX.log`. BSD `mktemp` replaces trailing `X`
characters, so a suffix after them left a literal shared filename. Parallel
tests collided before the wrapper could invoke `npx`.

**RC2 — the test asserted an input spelling, not the product value.**
`client_integrations.rs` intentionally canonicalizes `FORMAL_AI_PROXY_LOG`. On
macOS `/var/folders/...` resolves through `/private/var/folders/...`; the old
test expected the raw spelling. A symlink-alias fixture now reproduces that
semantic mismatch on Linux as well.

**RC3 — a Linux command line was mistaken for a portable interface.** The two
PTY tests embedded util-linux `script -qfec`. BSD `script` instead accepts a
transcript file followed by the command and its argv. The product never invokes
`script`; this was duplicated test infrastructure.

**RC4 — an empty Bash array is version-sensitive under nounset.** Stock macOS
Bash 3.2 can report an unbound variable for an empty `"${dests[@]}"` expansion
under `set -u`. The script had already enabled `nullglob`, making the empty
destination state both ordinary and reachable, but expanded it unconditionally.

**RC5 — Linux-only CI could not falsify any of RC1–RC4.** The test job contained
only `ubuntu-latest`, even though the repository already releases macOS desktop
artifacts. The platform whose utilities define these contracts never ran them.

**RC6 — the new runner leg inherited a GNU-only bootstrap command.** The shared
disk cleanup step used `df --output=avail`, which BSD `df` rejects with exit 64.
That step runs before toolchain setup, so the first fresh macOS job correctly
failed before it could exercise any Rust test. A fake BSD-shaped `df` makes the
bootstrap defect reproducible on Linux.

**RC7 — the interactive PTY fixture had two input-lifecycle races.** The
util-linux implementation queued the early `hi\n`, but BSD `script` could
process terminal EOF before the fake Codex client reached its read. Waiting for
`TUI_READY` removed that ordering race, but closing `script`'s stdin immediately
after the write still translated the pipe close into terminal EOT ahead of the
buffered line. The helper now uses a line-ending-independent readiness token
and retains the stdin handle until the PTY command exits.

## 5. Research and prior art

- The [FreeBSD `mktemp(1)` manual](https://man.freebsd.org/mktemp%281%29)
  documents the template as ending in `X` characters. Moving `.log` before the
  placeholder preserves the readable name without adding another command.
- The [FreeBSD `script(1)` manual](https://man.freebsd.org/script%281%29) gives
  the BSD shape as `file [command ...]`. A small shared Rust helper centralizes
  that dialect choice and retains util-linux `-qfec` on Linux.
- The [FreeBSD `df(1)` manual](https://man.freebsd.org/df%281%29) identifies
  `-k` and `-P` as POSIX options. `df -Pk /` therefore supplies a stable
  available-kilobytes column on both GNU and BSD implementations.
- GitHub’s maintained
  [runner-image table](https://github.com/actions/runner-images#available-images)
  lists `macos-15-intel`; the repository’s desktop-release matrix already uses
  the same label. This avoids adding a second macOS image convention.
- No PTY crate is already used by these tests. Adding a dependency for two
  test invocations would broaden the change while still leaving the packaging
  and Bash contracts dependent on real BSD utilities. The shared `script`
  adapter is the smaller boundary.
- `${dests[@]-}` was considered for Bash 3.2. An explicit length guard was
  chosen because it cannot synthesize an empty loop element and states exactly
  when array expansion is safe.

No third-party source or data is copied into the repository. The external
manuals and runner inventory are linked as behavioral references only.

## 6. Tests-first reproduction

Before changing production scripts or affected tests:

```console
$ cargo test --test issue_961_macos_portability -- --nocapture
running 5 tests
test package_log_uses_a_bsd_portable_mktemp_template ... FAILED
test proxy_log_expectation_matches_the_canonicalized_product_path ... FAILED
test pty_tests_do_not_embed_util_linux_only_script_flags ... FAILED
test seed_sync_guards_an_empty_destination_array ... FAILED
test complete_macos_portability_contract_holds ... FAILED
test result: FAILED. 0 passed; 5 failed
```

The original macOS audit supplies the platform execution proof: three parallel
packaging tests collided, two PTY tests received `script: illegal option -- f`,
the proxy-log expectation missed `/private/var`, and seed sync reported
`dests[@]: unbound variable` before its orphan pass.

The first pushed macOS job then supplied a second tests-first cycle for its
bootstrap dependency. Before changing `free-runner-disk.sh`, both
`macos_test_bootstrap_uses_portable_df_syntax` and
`runner_disk_cleanup_accepts_a_bsd_shaped_df` failed with
`df: unrecognized option '--output=avail'`, matching CI exactly.

The second pushed macOS job reached all 330 integration cases and failed only
the interactive wrapper case because `hi\n` was written before `TUI_READY`.
Before changing the helper and caller,
`pty_input_waits_for_tui_readiness_before_writing` failed. The helper unit test
also exercises a `TUI_READY\r\n` stream so the repaired synchronization does
not reintroduce a platform-specific newline assumption.

The third pushed macOS job again passed 329 of 330 integration cases, but its
captured order was `TUI_READY`, `^D`, then `hi`. The behavioral
`interaction_keeps_stdin_open_until_the_pty_command_exits` test failed against
the immediate-close helper with `stdin reached EOF before the PTY command
exited`, then passed after stdin's lifetime was extended through child exit.

## 7. Implemented fix

| Requirement | Change |
| --- | --- |
| R961-1 | `package-macos-with-retry.sh` now uses `formal-ai-macos-package.log.XXXXXX`, with every placeholder character at the end. |
| R961-2 | The integration fixture writes through a symlink alias, canonicalizes the resulting file, and compares diagnostics with that canonical path. |
| R961-3 | `tests/integration/pty.rs` selects BSD argv on macOS and a safely shell-quoted util-linux command elsewhere; both former call sites use it, and the interactive helper waits for `TUI_READY`, sends input, and defers EOF until the command exits. |
| R961-4 | `sync-seed.sh` checks `${#dests[@]}` before expanding the array, preserving the orphan loop for non-empty destinations. |
| R961-5 | The release test matrix now contains `ubuntu-latest` and `macos-15-intel`, with a 35-minute macOS budget and the existing 25-minute Linux budget. Its disk-cleanup bootstrap reads available space through POSIX `df -Pk` rather than GNU `--output`. |
| R961-6 | `tests/issue_961_macos_portability.rs` holds per-requirement and whole-task contracts; issue and PR evidence are committed separately. |

## 8. Formal AI / Agent CLI authorship

The reviewed decomposition is
[`issue-961-task-decomposition.lino`](issue-961-task-decomposition.lino). It has
seven smallest leaves. Formal AI, driven through the real Agent CLI, authors the
changelog fragment and the reviewed decomposition (2/7 = 28.6%, above the 20%
floor). The five platform code/test/CI leaves are manual because they extend the
test harness at the exact tool gap being investigated.

The two live session bundles under `self-hosting-authorship/` contain the real
server trace, Agent CLI log, and generated artifact. The canonical files are
byte-for-byte copies and are pinned by the issue regression test. Reproduce both
sessions with:

```bash
experiments/issue_961_self_authoring/run.sh
```

| Leaf | Agent CLI session | Evidence bundle |
| --- | --- | --- |
| Changelog fragment | `ses_014a75079ffewrf0nGhHCIG06f` | `self-hosting-authorship/changelog-session/` |
| Reviewed decomposition | `ses_014a725dbffeJQOYwc4T0yJjFT` | `self-hosting-authorship/decomposition-session/` |

The changelog intentionally uses the documented frontmatter-free form: the
release tooling defaults such a fragment to a patch bump, and the canonical
copy remains wholly Agent-authored.

## 9. Verification

Focused Linux verification after the fix:

```console
$ cargo test --test issue_961_macos_portability -- --nocapture
test result: ok. 12 passed; 0 failed
$ cargo test --test issue_757_session_files -- --nocapture
test result: ok. 2 passed; 0 failed
$ cargo test --test integration pty:: -- --nocapture
test result: ok. 4 passed; 0 failed
$ cargo test --test integration with_formal_ai::with_formal_ai_default_interactive_mode_launches_every_tool_in_a_pty -- --nocapture
test result: ok. 1 passed; 0 failed
```

The two original PTY integrations, packaging retry unit tests, complete local
suite, formatting, Clippy, examples check, repository integrity gates, and the
new macOS CI leg are recorded in PR #987 and its case study.
