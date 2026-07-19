# Issue #796 — Timeline, root causes, and solution plans

Evidence for every claim below is committed under this folder: full CI logs in
`ci-logs/`, GitHub API payloads and dependency traces in `raw/`.

## 1. What the issue reported

The issue listed four default-branch runs, two of them not passing:

| Workflow | Conclusion | Run |
| --- | --- | --- |
| Desktop Release | skipped | [29686858791](https://github.com/link-assistant/formal-ai/actions/runs/29686858791) |
| Desktop Release | skipped | [29686071628](https://github.com/link-assistant/formal-ai/actions/runs/29686071628) |
| Desktop Release | **failure** | [29681451142](https://github.com/link-assistant/formal-ai/actions/runs/29681451142) |
| CI/CD Pipeline | **failure** | [29680908415](https://github.com/link-assistant/formal-ai/actions/runs/29680908415) |

All four ran against the same commit, `12a4b34e` (the merge of PR #794).

The two `skipped` runs are **not defects**. Both are `workflow_run`-triggered
Desktop Release runs whose `resolve` job determined no desktop assets were
needed; every job reports `skipped`, and the run conclusion is `skipped`, not
`failure`. They are listed here for completeness and require no change.

## 2. Reconstructed sequence of events

```
09:03:40  Run 29680908415 (CI/CD Pipeline, push of merge commit 12a4b34e) starts
09:04:55  Detect Changes -> success
09:09:28  E2E, coverage, lint jobs -> success
09:21:31  Auto Release: 16 changelog fragments -> bump=minor
09:21:56  check-release-needed.rs -> should_release=true, 0.298.1 -> 0.299.0
09:22:01  version-and-commit.rs -> "Error recording self-hosting release metric:
          commit 59650f2b... must record both Formal-AI-Session and
          Formal-AI-Evidence"  -> exit 1                       [ROOT CAUSE C]
09:22:10  Run 29681451142 (Desktop Release) starts on the same commit
09:22:59  "Package VS Code extension (.vsix)" fails installing deps [ROOT CAUSE A]
09:26:37  Build macos-arm64 fails the same way                    [ROOT CAUSE A]
09:30:31  Build macos-x64 fails the same way                      [ROOT CAUSE A]
          Build windows-x64 / linux-x64 fail the same way         [ROOT CAUSE A]
11:56:57  Run 29686071628 (Desktop Release) -> skipped (no assets needed, benign)
12:22:26  Run 29686858791 (Desktop Release) -> skipped (no assets needed, benign)
```

Two independent root causes, landing within twenty minutes of each other on the
same commit, produced what looked like one broad CI collapse.

## 3. Root cause A — version-pinned npm deprecation allowlist

**Symptom.** Every job that installs Node dependencies fails with exit code 1
*after* npm itself succeeded (`added 587 packages in 21s`):

```
Unexpected npm stderr; update dependencies or explicitly classify the diagnostic:
npm warn deprecated glob@10.5.0: Old versions of glob are not supported, ...
##[error]Process completed with exit code 1.
```

Evidence: `ci-logs/run-29681451142.log` lines 377 (vsix) and 1500 (macos-arm64).

**Root cause.** `scripts/install-node-dependencies.sh` classified reviewed
upstream deprecations by **exact `name@version`**:

```bash
*"npm warn deprecated glob@7.2.3:"*|\
```

Transitive dependency versions float without any change on our side. When
`archiver-utils` resolved `glob` to `10.5.0`, the warning no longer matched any
allowlist entry, fell through to the "unexpected" bucket, and failed the build.

This is a textbook **false positive**: a warning about a package we do not
control, on a successful install, failing a release.

**Why now.** Upstream deprecated every `glob` below 12.x — including `10.5.0`
and `11.1.0`, which are themselves the patch releases for CVE-2025-64756. The
resulting confusion is tracked at
[isaacs/node-glob#644](https://github.com/isaacs/node-glob/issues/644), with
downstream fallout in
[mocha#5779](https://github.com/mochajs/mocha/issues/5779),
[jest#15910](https://github.com/jestjs/jest/issues/15910) and
[vscode#267530](https://github.com/microsoft/vscode/issues/267530).
So the warning is unavoidable noise, not an actionable vulnerability.

**Corrected attribution.** The original allowlist implied `glob` came from
electron-builder or vsce. Verified against the committed lockfiles, it does not:

```
@link-assistant/web-capture@^1.10.10
  -> archiver@7.0.1
    -> archiver-utils@5.0.2  (declares glob ^10.0.0)
      -> glob@10.5.0
```

Present in both `vscode/package-lock.json` and `desktop/package-lock.json`
(`raw/glob-dependency-chain.txt`). `@vscode/vsce@3.9.2` already depends on
`glob ^13.0.6`; electron-builder pulls only `glob@7.2.3`. Because
`@link-assistant/web-capture` is a first-party package, this one is fixable at
source rather than only suppressible — see the upstream report in §6.

**Fix applied.** `scripts/install-node-dependencies.sh` now matches reviewed
deprecations by **package name only**, so version floats cannot break CI, with
accurate per-package attribution URLs. Unreviewed diagnostics still fail the
build, so the check keeps its value. A verbose mode
(`INSTALL_NODE_DEPENDENCIES_VERBOSE=1`, **off by default**) traces every
classification decision.

**Alternatives considered and rejected.**

- `npm install --loglevel=error` would suppress the lines wholesale. Rejected:
  it also hides genuine diagnostics, and `tests/unit/ci-cd/issue_730.rs`
  explicitly asserts the script does not do this.
- Per-package suppression in npm does not exist; npm warns for the whole tree
  with no transitive-only filter
  ([npm/cli#7633](https://github.com/npm/cli/issues/7633)).
- No existing library or Action classifies npm stderr diagnostics; `npm ls
  --json` and `npm query` expose no `deprecated` field, so a regex over stderr
  remains the only mechanism.

## 4. Root cause B — desktop builds

Not an independent defect. All four `Build <platform>` jobs and the `vscode`
job fail through the same `install-node-dependencies.sh` call, so root cause A
fixes them all. This is why the fix had to be applied in the shared script
rather than per workflow.

## 5. Root cause C — a blank line hides a git trailer

**Symptom.** Auto Release fails immediately after computing the version:

```
Final release version: 0.299.0
Error recording self-hosting release metric: commit 59650f2b... must record
both Formal-AI-Session and Formal-AI-Evidence
```

Evidence: `ci-logs/run-29680908415.log` line 18713.

**The commit does record both.** From `raw/commit-59650f2b-message.txt`:

```
Formal-AI-Session: ses_089c27072ffeuapUqeaNfO3roA
                                <-- blank line
Formal-AI-Evidence: docs/case-studies/issue-751/.../agent-cli.log
```

**Root cause.** `scripts/self-hosting-metric.rs` read trailers via git's
`%(trailers:key=...)` placeholder. Per
[git-interpret-trailers](https://git-scm.com/docs/git-interpret-trailers),
trailers are extracted from *"a group of one or more lines"* that *"must either
be at the end of the input or be the last non-whitespace lines before a line
that starts with `---`."*

Only **one** group is recognised, and it must be the last one. The blank line
splits the two trailers into separate paragraphs, so git returns only
`Formal-AI-Evidence`. The script then saw a non-empty evidence list with an
empty session list — the exact shape of its "must record both" error — and
`version-and-commit.rs` treats that as fatal, failing the entire release.

Reproduced in a clean fixture repository:

```
SESSION=[]                       <-- present in the message, invisible to git
EVIDENCE=[docs/e.log]
```

So this is a **false negative** in trailer detection escalating into a false
positive failure: a correctly-annotated commit was reported as non-compliant.

**Fix applied.** `trailer_values` now scans the whole commit body (`%B`) for
`Key: value` lines, case-insensitively per git's own key semantics. Trailer
placement no longer determines whether a compliant commit is recognised.

**Trade-off.** Scanning the whole body is slightly more permissive than git's
last-paragraph rule — a trailer-shaped line in prose would now count. That is
the right direction for this check: the failure mode being fixed is silently
*dropping* valid attribution and blocking releases, and git's 25% heuristic
makes the placeholder unpredictable in the other direction too.

## 6. Requirements from the issue, and status

| # | Requirement | Status |
| --- | --- | --- |
| R1 | Fix failing Desktop Release run 29681451142 | Fixed (root cause A) |
| R2 | Fix failing CI/CD Pipeline run 29680908415 | Fixed (root cause C) |
| R3 | Check for false positives | Both defects were false positives; documented above |
| R4 | Check for false negatives | Root cause C was a false negative in trailer detection |
| R5 | Check warnings/errors | npm deprecations now surface as annotations, not failures |
| R6 | Apply fixes everywhere the problem occurs | Both fixes are in shared scripts used by every affected job; verified no other call site pins versions |
| R7 | Compare against the three pipeline templates | See `template-gap-analysis.md` |
| R8 | Report issues upstream where applicable | See §7 |
| R9 | Add debug output / verbose mode, default off | `INSTALL_NODE_DEPENDENCIES_VERBOSE`, off by default, test-enforced |

## 7. Upstream reports to file

1. **`@link-assistant/web-capture`** — bump `archiver` (or override `glob` to
   13.x) to clear `glob@10.5.0` from both workspaces at source.
2. **All three pipeline templates** — the Rust template's `release.yml` has no
   top-level `permissions:` block, so jobs inherit the repository default
   `GITHUB_TOKEN` scope. formal-ai fixes this with `permissions: contents:
   read`.
3. **All three templates** — `cargo install rust-script` is unretried;
   formal-ai's `scripts/install-rust-script.sh` short-circuits, uses `--locked`
   and retries with backoff.
4. **Rust/Python templates** — `codecov/codecov-action` pinned to `@v5`/`@v4`
   (v4 deprecated); formal-ai is on `@v7`.

## 8. Regression coverage

- `tests/unit/ci-cd/issue_796.rs` — five tests driving the real script against
  a fake `npm`: the floated `glob@10.5.0` line no longer fails; unreviewed
  diagnostics still do; scoped names keep their scope; verbose mode exists and
  is off by default; the allowlist may not contain a pinned version.
- `tests/unit/specification/self_hosting_metric.rs` — a commit whose trailers
  are separated by a blank line is attributed correctly.

Both suites were confirmed to **fail against the pre-fix scripts** and pass
after, with the trailer test reproducing the production error string verbatim.
