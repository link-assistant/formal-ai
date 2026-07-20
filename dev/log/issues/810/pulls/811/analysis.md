# Issue #810 — CI/CD false positives, false negatives, warnings and errors

Session: `issue-810-claude-20260720`
Pull request: https://github.com/link-assistant/formal-ai/pull/811

## 1. Evidence collected

| Artifact | Path |
| --- | --- |
| Desktop Release run 29738571290 (full log) | `ci-logs/run-29738571290.log` |
| Desktop Release run 29738571290 (job metadata) | `ci-logs/run-29738571290.json` |
| CI/CD Pipeline run 29737218421 (full log) | `ci-logs/run-29737218421.log` |
| CI/CD Pipeline run 29737218421 (job metadata) | `ci-logs/run-29737218421.json` |

Both runs are on the default branch at `bd0bac88` (the merge of #809), the exact
commit named in the issue body.

Failing jobs:

| Run | Job | Conclusion |
| --- | --- | --- |
| 29737218421 (CI/CD Pipeline) | Auto Release | failure |
| 29738571290 (Desktop Release) | Build macos-arm64 | failure |
| 29738571290 (Desktop Release) | Build macos-x64 | failure |
| 29738571290 (Desktop Release) | Publish SHA256SUMS.txt + provenance | failure (cascade) |

Every other job in both runs passed. There are exactly **two** independent
defects; the third failure is a downstream consequence of the second.

## 2. Defect A — the self-hosting evidence gate deadlocked every release

### Symptom

`ci-logs/run-29737218421.log:27173`:

```
Auto Release  Collect changelog and bump version
  Error recording self-hosting release metric: no committed Formal-AI-Evidence
  in 10e65ae2a22f206589ecba9974c95151e0124bc3 records session issue-804-claude-20260720
  ##[error]Process completed with exit code 1.
```

### Root cause

`scripts/self-hosting-metric.rs` attributes a commit to Formal AI only when the
document named by its `Formal-AI-Evidence` trailer contains the literal session
id from its `Formal-AI-Session` trailer (`commit_has_formal_ai_evidence`, around
line 142). Commit `10e65ae2` records:

```
Formal-AI-Session: issue-804-claude-20260720
Formal-AI-Evidence: dev/log/issues/804/pulls/805/analysis.md
```

but that document never spells its own session id out. Verified locally:

```
$ git show 10e65ae2:dev/log/issues/804/pulls/805/analysis.md | grep -c issue-804-claude-20260720
0
```

The check therefore returns `Err`, and `measure()` propagates it, aborting the
whole `Auto Release` job.

### Why it was not caught before merge — the timeline

1. `10e65ae2` is authored and merged as part of PR #805. The pull-request run
   for that branch (`29722531797`, head `8f5edf3a`) shows **no**
   `Self-Hosting Evidence Check` job in its job list — the gate did not exist
   yet.
2. `39fdef91` ("gate evidence trailers in PRs") adds the `evidence-check` job to
   `.github/workflows/release.yml`. `git merge-base --is-ancestor` confirms
   `39fdef91` lands **after** `10e65ae2`.
3. `10e65ae2` is now immutable history on `main`.
4. `record_release` measures `<last tag>..HEAD`. `10e65ae2` sits inside that
   range and will keep sitting there **until a new tag is cut**.
5. Cutting a new tag requires `Auto Release` to succeed. `Auto Release` aborts on
   `10e65ae2`.

That is a closed loop: the release cannot run because of a commit, and the only
thing that would move the range past that commit is a successful release. Every
`Auto Release` on `main` fails forever, with no possible fix in new commits.

This is a **false negative turned permanent outage**: a policy gate applied
retroactively to history that predates it.

### Fix applied

`scripts/self-hosting-metric.rs` grows an explicit `EvidencePolicy`:

- `Strict` — used by `measure()`, i.e. the pull-request `evidence-check` gate. A
  malformed evidence record is a hard error, rejected *before* it can reach the
  default branch. Enforcement is unchanged where enforcement is actionable.
- `Lenient` — used by `record_release()`. A malformed record is reported on
  stderr as `warning: not attributing <sha>: <reason>` and the commit simply is
  not counted as self-authored. Its lines still count toward `changed_lines`.

The metric can therefore only ever *under*-report self-hosting, never
over-report it, so the ratchet stays honest while a release can no longer be
held hostage by history.

### Regression test

`tests/unit/specification/self_hosting_metric.rs::a_malformed_historical_evidence_record_cannot_deadlock_a_release`
builds a fixture repo containing exactly the shape of `10e65ae2` (evidence that
mentions `formal-ai` but never names its session), then asserts both halves:
`measure()` still returns `Err`, and `record_release()` succeeds with
`self_authored_commits == 0` and `commits == 1`.

```
test specification::self_hosting_metric::a_malformed_historical_evidence_record_cannot_deadlock_a_release ... ok
test result: ok. 5 passed; 0 failed
```

### Follow-up worth considering

The gate would be more useful if it *generated* rather than *checked*: have the
solver template write `Formal-AI-Session: <id>` into the evidence document
itself, so the trailer and the document cannot drift apart. Requiring a human or
an agent to duplicate an identifier in two places is the actual defect generator
here.

## 3. Defect B — macOS ad-hoc signing breaks the bundled Chrome framework seal

### Symptom

`ci-logs/run-29738571290.log:6976` (identically at `:14883` for `macos-x64`):

```
electron-osx-sign Executing... codesign --sign - --force --timestamp=none
  --options runtime --entitlements build/entitlements.mac.plist
  .../formal-ai Desktop.app/Contents/Resources/browser-runtime/Frameworks/Google Chrome for Testing Framework.framework
> Stderr: ...: replacing existing signature
         ...: unsealed contents present in the root directory of an embedded framework
  ⨯ Command failed: codesign ...   failedTask=build
```

`Publish SHA256SUMS.txt + provenance` then fails as designed
(`log:16680`: "No artifacts from: macos-x64 macos-arm64") — that job is behaving
correctly and needs no change.

### What the evidence does and does not establish

Established:

- The custom sign hook **was** resolved and invoked. `log:4028` carries
  electron-builder's own marker, emitted only on the `customSign` branch of
  `MacPackager.doSign` (`app-builder-lib@26.15.3/out/macPackager.js:334`):
  `• executing custom sign  file=release/mac-arm64/formal-ai Desktop.app`.
- `--timestamp=none` on every `codesign` line confirms our `optionsForFile`
  override was in effect, so `signAsync` was called from our module.
- The signing path that failed is the one the hook is specifically written to
  skip.

Not established, and this is the blocker:

- **Not a single `[adhoc-sign-mac]` line appears anywhere in the 3.9 MB log**
  (`grep -ac "hook entered" → 0`), even though the workflow sets
  `FORMAL_AI_MACOS_SIGN_DEBUG=1` and the hook's very first statement is an
  unconditional banner. So we cannot tell whether `signingIgnoreRules()`
  returned a predicate that was consulted and answered "sign it", or was never
  consulted at all.

The two candidate root causes are therefore still open:

1. **Output loss.** `process.stderr.write()` on a pipe — which is what GitHub
   Actions hands a child process — is asynchronous and buffered. When
   electron-builder aborts the process on the signing error, anything still
   queued is discarded. `debug`-based output from `electron-osx-sign` survived,
   which is consistent but not conclusive.
2. **The predicate genuinely did not match.** Ruled unlikely: the exported
   `signingIgnoreRules` is unit-tested against the exact failing path, and
   `@electron/osx-sign@1.3.3`'s `validateOptsIgnore` (`dist/cjs/sign.js:52`)
   wraps a bare function into `[fn]` correctly — it is only the *array* form
   that it silently drops, which is the bug #808 already worked around.

### Instrumentation added (default state unchanged)

Per the "add debug output so the next iteration can find it" instruction:

- `log()` now writes through `fs.writeSync(2, ...)` instead of
  `process.stderr.write()`. A synchronous write to fd 2 cannot be lost to an
  aborting process. This eliminates candidate 1 outright: if the banner is still
  missing next run, the module is not the one executing.
- `signingIgnoreRules()` attaches a `stats` counter to the predicate, and the
  hook reports `ignore predicate: considered=N skipped=M` in a `finally` block —
  so it is emitted on failure as well as success. `considered=0` and
  `skipped=0` mean very different things and one line now distinguishes them.
- The per-file `ignore SKIP/sign ...` trace stays behind
  `FORMAL_AI_MACOS_SIGN_DEBUG`, **off by default**, exactly as before. Only the
  two summary lines are unconditional.
- Regression test:
  `desktop/scripts/adhoc-sign-mac.test.cjs::the ignore predicate counts what it
  considered and what it skipped`. Full desktop sign suite: 4 passed, 0 failed.

### Candidate fixes for the next iteration, in order of preference

1. **Confirm and fix the ignore path** once the counters land. If
   `skipped == 0`, the predicate never ran and the fix is in how the hook passes
   `ignore`; if `skipped > 0` but the runtime was still signed, `signAsync` is
   signing it from `opts.binaries` rather than the walked children.
2. **Do not hand codesign the framework at all.** Ship the browser runtime as an
   archive under `Contents/Resources` and expand it on first launch. `codesign`
   never walks into it, the upstream Chrome signature is preserved byte for
   byte, and the failure mode disappears structurally rather than by
   configuration. Cost: a first-run extraction step and its error handling.
3. **Move the runtime out of the app bundle** into `Application Support`,
   downloaded or expanded on demand. Smallest bundle, largest behavioural
   change.

Option 2 is the one that removes the class of bug rather than the instance —
"unsealed contents present in the root directory of an embedded framework" is
`codesign` correctly refusing to co-sign a bundle it did not lay out, and no
amount of ignore-rule tuning changes that. Worth doing regardless of what the
counters show.

### Upstream reporting

No upstream issue is warranted yet. The `@electron/osx-sign` array-vs-function
`ignore` bug is real and already documented in-repo at
`desktop/scripts/adhoc-sign-mac.cjs`, but it is a known upstream behaviour that
#808 worked around; a report should wait until the counters tell us whether the
remaining failure is upstream's or ours. Filing now would be filing a guess.

## 4. Requirement checklist from the issue

| # | Requirement | Status |
| --- | --- | --- |
| 1 | Collect all logs/data into `dev/log/issues/810/pulls/811` | Done — 10 MB of run logs plus job metadata |
| 2 | Deep analysis, timeline, root cause per problem | Done — §2 and §3 |
| 3 | Fix false positives / false negatives / warnings / errors | Defect A fixed and tested; Defect B instrumented, not yet fixed |
| 4 | Add debug output + verbose mode, default off, when data is short | Done — §3, per-file trace remains behind `FORMAL_AI_MACOS_SIGN_DEBUG` |
| 5 | Report upstream issues where applicable | Assessed, deliberately deferred — §3 |
| 6 | Apply each fix everywhere it occurs | Defect A: single call site (`record_release`), verified by grep across `scripts/*.rs`. Defect B: single hook, shared by both mac jobs |
| 7 | Compare against the three CI/CD pipeline templates | **Not done** — see §5 |

## 5. Known gap

Requirement 7 (a file-by-file comparison against
`link-foundation/{rust,js,python}-ai-driven-development-pipeline-template`, and
reporting anything found back to those templates) has not been carried out in
this pass. Neither of the two observed failures originates in template-derived
workflow structure — Defect A is repo-specific policy tooling and Defect B is
repo-specific desktop packaging — so the template sweep is genuinely separate
work rather than a prerequisite for the fixes above. It should not be marked
complete until someone diffs the workflows.

---

## 6. Second iteration — two nondeterministic failures on the PR branch

Run [29742025207](https://github.com/link-assistant/formal-ai/actions/runs/29742025207)
(head `70a09859`) failed two jobs that pass on `main`. Neither is caused by the
branch's diff (which touches only `scripts/self-hosting-metric.rs` and its test):
both are **flakes**, i.e. false negatives, which requirement 3 of the issue
covers explicitly. Evidence: `ci-logs/coverage.log`, `ci-logs/e2e.log`,
`ci-logs/formal-ai-serve-8776.log`, `ci-logs/agent-out-8776.log`. The six most
recent `main` runs fail only in `Auto Release` (Defect A), never in these jobs.

### Defect C — `desktop_release_resolve` unit test, ETXTBSY on the mock `gh`

```
ci_cd::desktop_release_resolve::workflow_run_skips_when_release_has_all_required_assets
  left: "true"   right: "false"
```

`1937 passed; 1 failed`, and the same test passes locally and in every earlier
run. The test writes a mock `gh` into a scratch `PATH` directory and immediately
executes it. The unit suite is multi-threaded: any `Command` spawn in another
test thread forks this process, and a fork landing between the mock's `write`
and its `close` inherits the still-open write descriptor. `execve` on an inode
that any process holds open for writing fails with `ETXTBSY`. The resolve script
swallows a failed `gh` (`|| true`), so the asset query degrades silently to "no
assets exist" and `should_build` flips to `true` — exactly the observed values.

Fixes:

* The mock is written to `gh.staging` and `rename`d into place, so the name that
  gets executed is never the file being written (`tests/unit/ci-cd/desktop_release_resolve.rs`).
* `scripts/desktop-release-resolve.sh` no longer conflates "`gh` failed" with
  "the release has no assets": it keeps the query's exit status and logs a
  warning. The decision is unchanged (both still build — the fail-safe
  direction); only the diagnosis improves.
* The assertion now prints the script's stdout, which carries the
  `existing desktop assets: N` / `missing required desktop assets` lines.

### Defect D — agent CLI E2E, auto-compaction swallowed the report prompt

```
!! report request did not execute gh
```

`formal-ai-serve-8776.log` shows the whole sequence. The research turn fetched
three live pages (fec.gov, usatoday, wikipedia) and grew the transcript to a
246 KB request. opencode then issued a *summarisation* request ("You are a
helpful AI assistant tasked with summarizing conversations"), and the next turn
arrived at the server with last user message `"Continue if you have next steps"`
— not `"Report this problem"`. The server logged
`agentic_outcome: fallthrough (task unrecognised)`, no `gh` ran, and the
assertion failed for a reason unrelated to the behaviour under test.

Root cause: the harness advertised a 200000-token context limit, and transcript
size depends on whatever the live pages happen to serve that day — so whether
compaction triggers is a property of the public web, not of this repository.

Fix (`experiments/agent_cli_e2e/run_issue_687.sh`): raise the advertised limit to
4000000. It is a harness knob only — the server never enforces it — so this
removes the live-content dependency rather than masking a product defect. A
diagnostic branch now also names compaction explicitly when the `gh` log is
missing and the server log contains a summarisation request, so the two failure
modes are never again confused.

## 7. Template sweep (requirement 7)

Cloned `link-foundation/{rust,js,python}-ai-driven-development-pipeline-template`
and diffed their workflows against `.github/workflows/{release,desktop-release}.yml`,
then re-checked both against
`link-assistant/hive-mind/docs/CI-CD-BEST-PRACTICES.md`.

Already matching the templates, verified line by line: concurrency groups with
`cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`, `GIT_CONFIG_*`
default-branch hint suppression, least-privilege `permissions: contents: read`
with per-job escalation, `timeout-minutes` on every job, `detect-changes`
fast-fail ordering, secrets scan, version-check, and attest-before-upload.

Acted on:

* **False negative — desktop dry run missed Rust changes.** `desktop-release.yml`'s
  pull-request `paths` filter listed only `desktop/**`, `vscode/**` and two
  scripts, yet the build job runs `cargo build --release --bin formal-ai` and
  bundles that binary. A Rust-only change could break desktop packaging with a
  green PR. `src/**`, `Cargo.toml` and `Cargo.lock` are now in the filter.

Checked and rejected:

* The `for f in release/latest*.yml; do [ -f "$f" ] && files+=("$f"); done` loops
  (`desktop-release.yml` collect/upload steps) were flagged as a `set -e` abort
  when the last file is absent. They are not: bash exempts a failing command
  that is part of an `&&` list. Verified directly —
  `bash -c 'set -euo pipefail; for f in a b; do [ -f "$f" ] && echo yes; done; echo ALIVE'`
  prints `ALIVE`.
* `finalize`'s `if: always() && … should_build == 'true'` was flagged as letting
  a failed `resolve` pass silently. It does not: a failed job fails the run
  regardless of what the downstream job does.

Still open (recorded, not done — each is a refactor rather than a fix):

* The js template's `check-file-line-limits` job covers `.md` and `release.yml`;
  our `scripts/check-file-size.rs` covers only `.rs`, `.lino` and worker `.js`.
  `release.yml` is 1791 lines, over the documented 1500-line ceiling, with
  nothing failing.
* `simulate-fresh-merge.sh` runs in `lint` and `test` but not in
  `desktop-release.yml`, so the packaging dry run builds the PR head rather than
  the merge result.
