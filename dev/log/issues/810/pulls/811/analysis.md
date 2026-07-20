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
