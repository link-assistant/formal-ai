# Case Study — Issue #736: Check for all false positives, false negatives, warnings and errors in CI/CD and fix them all

- **Issue:** [link-assistant/formal-ai#736](https://github.com/link-assistant/formal-ai/issues/736)
- **Pull request:** [link-assistant/formal-ai#737](https://github.com/link-assistant/formal-ai/pull/737)
- **Failing runs analysed:** [29485000765](https://github.com/link-assistant/formal-ai/actions/runs/29485000765),
  [29484631709](https://github.com/link-assistant/formal-ai/actions/runs/29484631709),
  [29312084458](https://github.com/link-assistant/formal-ai/actions/runs/29312084458),
  [29214051612](https://github.com/link-assistant/formal-ai/actions/runs/29214051612)
- **Dates of failure:** 2026-07-12 → 2026-07-16
- **Author of analysis:** AI issue solver

This folder contains the raw evidence ([`./data`](./data)) and the deep analysis of
every CI/CD defect surfaced by issue #736: the root cause of each, the fix applied,
and the upstream reports filed where the defect is not ours to fix.

---

## 1. Executive summary

The `Auto Release` job is the only job that turned any of the four analysed runs
red — but it failed for **two entirely different reasons**, which is why the
failure looked intermittent and resisted a single explanation:

| # | Defect | Severity | Evidence | Verdict |
|---|--------|----------|----------|---------|
| 1 | **Auto Release — `cannot rebase: Your index contains uncommitted changes`** | ❌ failure | runs 29214051612, 29484631709 | **Real bug in `scripts/version-and-commit.rs`** — fixed in this PR |
| 2 | **Auto Release — `No space left on device`, runner killed** | ❌ failure, **silent** | runs 29312084458, 29485000765 | **Real infrastructure failure** — fixed in this PR |
| 3 | **docs.rs "All builds failed"** ("docs generation shows as failing") | ❌ failure | docs.rs build 3868612 | **Upstream bug** in `lindera-jieba` — reported, not fixable here |
| 4 | **`desktop-release.yml` publishes partial releases as green** | ⚠️ **false positive** | code read | **Real bug** — fixed in this PR |
| 5 | **`desktop-release.yml` uploads Linux/Windows artifacts unverified** | ⚠️ **false negative** | code read | **Real gap** — fixed in this PR |
| 6 | **`desktop-release.yml` runs show as `skipped`** | ℹ️ cosmetic | job `if:` at `desktop-release.yml:80-84` | **Correct behaviour**, not a defect — see §4.6 |
| 7 | **Every release writes a `CHANGELOG.md` the reconstruction check rejects** | ❌ failure, **latent** | reproduced against `b2064b2a` | **Real bug** in both release writers — fixed in this PR, see §4.7 |
| 8 | **A test asserts a changelog fragment exists forever; releases consume fragments** | ❌ failure, **latent** | jobs 87682984589, 87682984539 | **Real bug** — fixed in this PR, see §4.8 |

Defects 1 and 2 are the serious ones, and defect 2 is worse than defect 1: by the
time the runner died, the crate had **already been published to crates.io**, so
those runs shipped a version with **no Docker image and no GitHub Release**.

Defects 7 and 8 were not visible in any of the four analysed runs. Both surfaced
only when this PR became the first change in a while to actually run the checks,
and **both had been red on `main` since the `v0.296.0` release commit**
(`b2064b2a`) — which is also the commit that caused them. They are the clearest
examples of what this issue asked for: defect 7 had been repeatedly *papered
over* by hand rather than fixed at source, and defect 8 was invisible because the
job that catches it does not run on the commits that break it.

Both share one root cause worth stating plainly: **`main` is not actually
verified.** `Detect Changes` path-filters the heavy jobs, and a release commit
touches none of the triggering paths — so a release can break the lint, test and
coverage jobs and still report ✅, because those jobs never ran. The breakage
lands on the next contributor's unrelated PR.

---

## 2. Timeline / sequence of events

All times UTC, from [`data/run-*.json`](./data) and [`data/job-*.json`](./data).

| Time | Event |
|------|-------|
| 2026-07-12 23:48:07 | Push to `main` (`2dcaeee4`) starts run `29214051612`. |
| 2026-07-13 00:17:36 | `Auto Release` (job `86709184220`) starts. Fails at **step 8, "Collect changelog and bump version"** — defect #1. |
| 2026-07-14 06:40:46 | Push to `main` (`64cb3b84`) starts run `29312084458`. |
| 2026-07-14 06:57:50 | `Auto Release` (job `87020644690`) starts. Fails with **no failed step at all** — the runner worker died writing its own diag log — defect #2. |
| 2026-07-16 08:45:55 | Merge of PR #731 (`6b1910a3`) starts run `29484631709`. |
| 2026-07-16 08:51:53 | Merge of PR #690 (`9052d2ca`) starts run `29485000765`, **6 minutes after the previous one and while it is still running**. |
| 2026-07-16 09:05:41 | `Auto Release` for the *older* run (job `87580040366`) starts. |
| 2026-07-16 09:07:33 | It logs `Local branch is behind remote, rebasing...` then `Error rebasing onto origin/main: Command failed: error: cannot rebase: Your index contains uncommitted changes.` Step 8 fails; steps 9–21 skip — defect #1. |
| 2026-07-16 09:21:58 | `Auto Release` for the newer run (job `87583325043`) starts and dies with `No space left on device` — defect #2. |

**The 08:45 / 08:51 pair is the reproduction of defect #1.** Two releases landing
on `main` six minutes apart is precisely the race the rebase bug needs: the older
run had already staged its version bump when the newer release's commit appeared
on `origin/main`, and `git rebase` refuses to run against a dirty index.

This also explains why the bug looked flaky rather than broken. Of the 25 most
recent `push` runs on `main`, only these four failed; the rest had no concurrent
push land inside their release window and so never rebased at all.

### 2.1 Why the evidence was hard to obtain

For jobs `87020644690` and `87583325043`, `gh run view --log` returns
`log not found` and the API log endpoint returns `BlobNotFound` / HTTP 404 (see
[`data/api-404-job-87583325043.txt`](./data/api-404-job-87583325043.txt)). **This is
not a tooling problem — it is a symptom of the defect itself.** The runner ran out
of disk while flushing its own log, so there is no log blob to download.

The evidence lives only in the job's **failure annotations**
([`data/annotations-87583325043.json`](./data/annotations-87583325043.json)), which
is the same lesson issue #523 recorded: for runner-level crashes, fetch
annotations, not step logs.

---

## 3. Requirements extracted from the issue

| ID | Requirement | Status |
|----|-------------|--------|
| **R1** | `release.yml` auto-release is failing → fix it | ✅ §4.1, §4.2, §4.7, §4.8 |
| **R2** | `desktop-release.yml` — double check everything works perfectly | ✅ §4.4–§4.6 |
| **R3** | Docs generation shows as failing → fix it | ✅ §4.3 (upstream; reported) |
| **R4** | Compare the full file tree against the js/rust/python/csharp pipeline templates; reuse best practices; report shared defects upstream | ✅ §5 |
| **R5** | Compile all logs/data into `./docs/case-studies/issue-736` and do a deep analysis: timeline, requirements, root causes, solution plans, known components/libraries | ✅ this document |
| **R6** | If there is not enough data to find the root cause, add debug output / verbose mode for the next iteration | ✅ §4.2 (`RUNNER_DISK_DEBUG`, low-disk annotation) |
| **R7** | Report issues to other affected repositories, with reproducible examples, workarounds and fix suggestions | ✅ §6 (4 upstream + [#738](https://github.com/link-assistant/formal-ai/issues/738) here) |
| **R8** | Apply each fix across the entire codebase — if the problem exists in multiple places, fix all of them | ✅ §4.2 (3 jobs), §4.4 (both attest sites), §4.7 (both release writers), §4.8 (grepped `tests/`, sole instance) |
| **R9** | Do everything in this single PR (#737) | ✅ |

---

## 4. Root-cause analysis

### 4.1 Defect #1 — `cannot rebase: Your index contains uncommitted changes`

**Evidence (verbatim, [`data/autorelease-87580040366.log`](./data/autorelease-87580040366.log) lines 1344-1345):**

```
Local branch is behind remote, rebasing...
Error rebasing onto origin/main: Command failed: error: cannot rebase: Your index contains uncommitted changes.
```

**Root cause.** `scripts/version-and-commit.rs` did its work in this order:

1. read the manifest and compute the new version,
2. write the bump into `Cargo.toml` / `CHANGELOG.md`,
3. **`git add`** those files,
4. *then* `git fetch` + `git rebase origin/main`.

Step 4 cannot work after step 3. `git rebase` hard-refuses to run against a dirty
index — that is documented, intended git behaviour, not an edge case. The ordering
was only ever exercised when the fetch found new commits, which needs a concurrent
push to land inside the release window; hence the intermittency.

Two adjacent defects in the same function, found while fixing this:

- **A mislabel.** The code rebased whenever `local != remote`, and reported that as
  `Local branch is behind remote`. Being *ahead* of the remote — the normal state
  right after committing a bump — is not being behind. Fixed by counting with
  `git rev-list --count HEAD..origin/<branch>`.
- **A tag-ordering hazard.** The tag was created *before* the push-retry loop, so a
  `pull --rebase` retry inside that loop could leave the tag pointing at an
  orphaned pre-rebase commit. Fixed by tagging only after the commit reaches the
  remote.

**Fix.** `sync_with_remote()` now runs **early, while the tree is still clean**,
before the manifest is even read — so the bump is computed from the newest state of
the branch rather than a stale checkout, and there is no index to trip over.

**Regression tests** (`scripts/version-and-commit.rs`, run in CI via
`rust-script --test`): `syncs_with_concurrent_release_then_commits_bump_on_top`
builds a real repo with a bare origin, pushes a concurrent commit, and asserts the
bump lands on top; `syncing_after_staging_the_bump_fails` pins the *old* ordering as
an error (asserts `cannot rebase`), so the bug cannot be reintroduced;
`does_not_rebase_when_only_ahead_of_remote` covers the mislabel.

### 4.2 Defect #2 — `No space left on device` (the silent one)

**Evidence (verbatim, [`data/annotations-87583325043.json`](./data/annotations-87583325043.json)):**

```
System.IO.IOException: No space left on device : '/home/runner/actions-runner/cached/2.335.1/_diag/Worker_20260716-092159-utc.log'
```

**Root cause.** The release jobs build the crate with `cargo build --release`, then
build the Docker image — and [`Dockerfile`](../../../Dockerfile) is a two-stage build
whose `rust:1.96-slim` builder runs `cargo build --release --locked` **again**, on
top of a large base image. Hosted `ubuntu-latest` ships ~14 GB free on `/`. Two
release compiles plus the image layers exceed it and take the runner process itself
down.

**This is the same failure as issue #523**, whose fix — reclaiming disk by removing
unused pre-installed SDKs — was applied **only to the Pages job**. The release jobs
never got it. That is exactly the "if we have an issue in multiple places it should
be fixed in all of them" case from R8.

**Why it is worse than defect #1.** The job dies *after* step 11 (`cargo publish`)
has already succeeded. So these runs pushed a version to crates.io and then
produced no Docker image and no GitHub Release — a **silent partial release**, with
no failed step and no downloadable log to explain it.

**Fix.** The reclaim now lives in the shared
[`scripts/free-runner-disk.sh`](../../../scripts/free-runner-disk.sh) and runs in
**all three** heavy jobs: auto-release, manual-release, and the Pages job (which
now calls the shared script instead of its inline copy).

**Debug output for the next iteration (R6).** The script prints `df -h` before and
after, reports the MiB reclaimed, supports `RUNNER_DISK_DEBUG=1` for a
per-directory breakdown, and — most importantly — emits a
`::warning title=Low runner disk::` annotation when less than `RUNNER_DISK_MIN_MIB`
(default 6144) remains. A future recurrence therefore leaves a diagnosable
annotation behind even when, as here, no log survives.

### 4.3 Defect #3 — "docs generation shows as failing" is docs.rs, not GitHub Actions

The repository has exactly **two** workflows (`release.yml`, `desktop-release.yml`)
and neither has a failing docs job. The failing docs generation is **docs.rs**,
which reports *All builds failed* for `formal-ai` 0.296.0 (build 3868612).

**Evidence (verbatim, [`data/docsrs-build-3868612-formal-ai-0.296.0.log`](./data/docsrs-build-3868612-formal-ai-0.296.0.log)):**

```
Compiling lindera-jieba v3.0.7
error: failed to run custom build command for `lindera-jieba v3.0.7`
  Error: LinderaError { kind: Io, source: Failed to create dummy input directory:
  ".../mecab-jieba-0.1.1" Caused by: File exists (os error 17) }
```

**Root cause — located exactly.** The failing code is not in `lindera-jieba`'s own
`build.rs`; that build script just calls `lindera_dictionary::assets::fetch()`. The
defect is in **`lindera-dictionary/src/assets.rs:284-293`**, and it is inside the
branch written specifically to make docs.rs work:

```rust
if std::env::var("DOCS_RS").is_ok() {
    // Create directory for dummy input directory for build docs
    fs::create_dir(&input_dir).map_err(|err| {         // ← not create_dir_all
        LinderaErrorKind::Io
            .with_error(anyhow::anyhow!(err))
            .add_context(format!("Failed to create dummy input directory: {input_dir:?}"))
    })?;
```

`fs::create_dir` is **not idempotent**: it returns `AlreadyExists` (errno 17) if the
directory is there. Three facts make that fatal on docs.rs:

1. With `LINDERA_DICTIONARIES_PATH` unset, `build_dir` is `OUT_DIR` and `is_cache`
   is `false` (`assets.rs:233-264`).
2. Because `is_cache` is `false`, the early-return fast path
   `if is_cache && output_dir.is_dir() { return Ok(()) }` (`assets.rs:279-282`) can
   **never** fire — so a re-run always reaches the `create_dir`.
3. The build script declares `cargo:rerun-if-env-changed=DOCS_RS`
   (`assets.rs:226`). When docs.rs builds the crate and then re-invokes with
   `DOCS_RS` set, cargo re-runs the build script **against the same `OUT_DIR`**,
   where `input_dir` already exists from the first run. `create_dir` → errno 17.

So the code that exists to support docs.rs is the code that breaks docs.rs.
`create_dir_all`, or tolerating `AlreadyExists`, is the fix.

**Reproduced locally.** [`experiments/lindera-docsrs-repro`](../../../experiments/lindera-docsrs-repro)
reproduces the docs.rs failure byte-for-byte with two `cargo build` runs, and in
doing so exposed a **second** bug that code reading alone had missed: the dummy
dictionary is scaffolded flat into `input_dir/`, ignoring `src_subdir`, while the
builder reads `input_dir/dict-src/` — which is what `lindera-jieba` sets. So the
*first* build in a fresh sandbox already fails:

```
Error: LinderaError { kind: Build, source: Failed to build dictionary
Caused by: LinderaError(kind=Io, source=Failed to open file:
  .../out/mecab-jieba-0.1.1/dict-src/char.def) }
```

docs.rs therefore has no green path at all: fresh builds fail on the missing
`dict-src/char.def`, re-runs fail on `File exists`.

**Upgrading lindera does not help.** `lindera-dictionary` 4.0.0 is the newest
version on crates.io and carries the **identical** `create_dir` at `assets.rs:286`
— verified by downloading and diffing both 3.0.7 and 4.0.0 from
`static.crates.io`. A `meta-language` bump to lindera 4.x would therefore *not*
fix docs.rs; only an upstream `lindera` fix, or dropping/feature-gating the
dependency, will.

**Bisected precisely.** formal-ai 0.183.0 builds green; 0.184.0 fails. The only
Cargo.toml change between them is `+meta-language = "0.39"`. The chain is
`formal-ai` → `meta-language` (`Cargo.toml:43`) → `lindera ^3.0.7` (non-optional,
`default_features = false, features = ["embed-jieba"]`) → `lindera-jieba`.

**Not fixable in this repository.** `meta-language` exposes only `default = []` and
`doublets`, so no feature selection here can drop `lindera`, and no
`[package.metadata.docs.rs]` setting would help. Proof that it is upstream:
`meta-language` 0.45.0 fails on docs.rs identically (build 3577282), with no
formal-ai involved.
Reported upstream — see §6.

### 4.4 Defect #4 — `desktop-release.yml` reports partial releases as complete (false positive)

The `finalize` job ran with `if: always() && ...`, failed **only when zero**
fragments existed, and hardcoded all six builders into `BUILD-PROVENANCE.txt`.
Because the build matrix uses `fail-fast: false`, a run where five of six
architectures failed still published a green, authoritative-looking
`SHA256SUMS.txt` **asserting builds that never happened**. This is the textbook
false positive the issue asks about: green means "all desktop assets shipped", and
callers rely on that.

**Fix.** The provenance now lists only builders that actually produced a fragment,
names the missing targets under an `INCOMPLETE :` heading, drops the word
"authoritative", and the job **fails after** uploading — the partial manifest is
still published for debugging, but the run ends red.

Two more defects fixed in the same file:

- **Attest-after-upload.** Both the desktop and vscode jobs uploaded assets and
  *then* attested them, leaving assets publicly downloadable without provenance for
  the length of the attest step — and permanently so if attest failed. Attestation
  now runs before the upload in **both** jobs (R8).
- **A concurrency group that never dedups.** The group read
  `github.event.release.tag_name || github.event.inputs.tag || github.run_id`, but
  the automatic `workflow_run` path carries **neither** of the first two, so it fell
  through to the always-unique `run_id`. Concurrent runs for the same tag therefore
  raced on `gh release upload --clobber` and on `finalize`'s `SHA256SUMS.txt`. Now
  keyed on `github.event.workflow_run.head_sha`, which that event does carry.

### 4.5 Defect #5 — Linux/Windows artifacts shipped unverified (false negative)

The smoke test was gated `if: matrix.ebflag == '--mac'`, so **four of the six**
targets were uploaded to the release with nothing checking that electron-builder
produced them under the expected names. A rename or a partial package would ship
silently — a false negative.

**Fix.** A `Smoke test Linux/Windows release artifacts` step verifies the expected
versioned artifacts exist and are non-empty for each Linux/Windows matrix leg. The
deep macOS checks (`hdiutil`, `codesign`, `spctl`, `stapler`) are macOS-only by
nature, but existence and non-emptiness are checkable everywhere.

### 4.6 Non-defect — `desktop-release.yml` runs showing as `skipped`

The repeated `skipped` runs visible in the issue's screenshot are **correct**. The
job's `if:` (`desktop-release.yml:80-84`) requires `head_branch == 'main'`, and the
skipped runs are `pull_request` events on issue branches. This is cosmetic list
noise, not a false positive, and suppressing it would mean weakening a guard that
does its job. **No change.**

Also checked and found sound: the required-asset count `17` in
`scripts/desktop-release-resolve.sh:230` matches `expected_desktop_assets()`.

### 4.7 Defect #7 — every release writes a `CHANGELOG.md` the check rejects

**Symptom.** `Lint and Format Check` → step 12, `Check reconstructed changelog`:

```
Error: CHANGELOG.md differs from reconstructed Git history
```

**This was already red on `main`.** Verified directly, against a clean worktree
of `origin/main` with nothing from this PR in it:

```bash
git worktree add --detach /tmp/mainwt origin/main
cd /tmp/mainwt && node experiments/issue_711_rebuild_changelog.mjs --check   # exit 1
```

**Why nobody saw it.** The run on this branch's base commit (`d6c34537`) is
listed as ✅ *success* — but its lint job was **`skipped`**, along with every
other real check. `Detect Changes` path-filters the lint job, and a release
commit does not touch the paths that trigger it. So the check that guards the
release artifacts *cannot run on the release that breaks them*. The red only
lands later, on the next unrelated PR — far from the change that caused it.

**Root cause — located exactly.** `scripts/version-and-commit.rs:462-466`. The
writer ignores the insert marker and splices the new section in before the first
`## [` line, which produces **two** defects at once:

```rust
let new_entry = format!("\n## [{}] - {}\n\n{}\n", version, date_str, ...);  // leading \n
// ...
new_lines.extend(lines[idx..].iter().map(|s| s.to_string()));
content = new_lines.join("\n");                                             // trailing \n lost
```

1. `lines[..idx]` **already ends with the blank line that follows the marker**,
   and `new_entry` opens with another `\n` → the marker is left followed by two
   blank lines. The canonical form built by `issue_711_rebuild_changelog.mjs`
   (`HEADER + "\n\n" + sections.join("\n\n") + "\n"`) allows exactly one.
2. `.lines()` **drops the trailing newline**, and `join("\n")` never restores it
   → every release strips the file's final newline.

Both predictions were confirmed byte-for-byte before any fix was written — first
against the real `v0.296.0` release commit:

```console
$ git show b2064b2a -- CHANGELOG.md
 <!-- changelog-insert-here -->
 
+                          ← the extra blank line
+## [0.296.0] - 2026-07-16
```

and then by the regression test, which failed with exactly the two predicted
differences and nothing else:

```
left:  "...insert-here -->\n\n\n## [0.2.0]...- Initial release"     ← 3 newlines, no trailing \n
right: "...insert-here -->\n\n## [0.2.0]...- Initial release\n"     ← canonical
```

**The tell was in the git history.** `fix: refresh reconstructed release
artifacts` appears over and over on `main` (`2ca32ad2`, `28a5b63a`, `facdd3c2`,
`5fb6e19d`). Each one is a person regenerating the artifacts by hand instead of
fixing the writer — a treadmill, once per release, for a three-line bug.

**Fix.** Both release paths — automatic (`version-and-commit.rs`) and manual
(`collect-changelog.rs`, which had the mirror-image defect: its marker branch
emitted *zero* blank lines) — now emit the canonical shape, pinned by
`release_writes_the_changelog_exactly_as_reconstruction_expects`, which compares
output byte for byte rather than with the `contains()` assertions the neighbouring
test uses (those pass on both the broken and the fixed output, which is why they
never caught this).

**Residual, reported as [#738](https://github.com/link-assistant/formal-ai/issues/738).**
The fix stops `CHANGELOG.md` diverging, but `fragment-release-map.tsv` still goes
stale on every release, so the treadmill continues for that one file. It is
**not fixable inside the release commit**: the map's third column is the SHA of
the release commit itself, which does not exist until the commit is made. That
needs a design decision (drop the SHA column, or amend post-commit) rather than a
patch smuggled into this PR — especially as the amend route would touch the
push/rebase path repaired in §4.1.

---

### 4.8 Defect #8 — a test asserts a changelog fragment exists forever

**Symptom.** Two jobs on this branch failed, and both failed on the *same single
test*:

```
---- docs_requirements_issue_656::issue_656_promotion_documents_are_traceable stdout ----
assertion failed: path.is_file()
changelog.d/20260714_090000_issue_656_promotion.md should exist for issue #656 traceability
        at tests/unit/docs_requirements_issue_656.rs:74
test result: FAILED. 1663 passed; 1 failed
```

`Test (ubuntu-latest)` (job `87682984589`) and `Code Coverage` (job
`87682984539`) both run the same suite, so one assertion took two checks down.

**Root cause.** The test listed the changelog fragment among files that "must
exist", but **changelog fragments are consumed by the release that ships them** —
that is their designed lifecycle. `changelog.d/*.md` is written by a contributor,
collected into a `CHANGELOG.md` section at release time, and deleted. The
`v0.296.0` release (`b2064b2a`) consumed exactly that fragment:

```console
$ git log --oneline -1 --diff-filter=D -- changelog.d/20260714_090000_issue_656_promotion.md
b2064b2a chore: release v0.296.0
```

So the assertion was not wrong when it was written — it was **correct only until
the next release**, and it has failed on every run since.

**Why nobody saw it.** Same mechanism as §4.7: the release commit that deletes
the fragment touches no path that trips `Detect Changes`, so `Test` and
`Code Coverage` never ran on it. The release reported ✅; the red surfaced later,
on an unrelated PR.

**Scope check (R8).** Grepped the whole of `tests/` for the same shape — a
`changelog.d/` path inside an existence assertion. This is the **only** test with
the flaw, so the fix is a one-file change rather than a sweep.

**Fix.** Follow the entry across its lifecycle instead of pinning it to one side:
before release it is a fragment, after release it is a `CHANGELOG.md` section.
Either satisfies the traceability intent, and the test now holds on **both** sides
of a release.

```rust
let fragment = root.join("changelog.d/20260714_090000_issue_656_promotion.md");
if fragment.is_file() {
    assert!(fragment.metadata().map_or(0, |meta| meta.len()) > 0, ...);
} else {
    assert!(
        read(root.join("CHANGELOG.md")).contains("issue #656"),
        "the issue #656 changelog fragment was consumed by a release, so \
         CHANGELOG.md must carry its entry for traceability",
    );
}
```

Verified locally: `test ... issue_656_promotion_documents_are_traceable ... ok`
(`1 passed; 0 failed; 1665 filtered out`).

---

## 5. Template comparison (R4)

Compared against
[rust-ai-driven-development-pipeline-template](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template)
(the applicable one; the js/python/csharp templates share the workflow *shape* but
not the Rust release script).

**Already fixed upstream, drifted here.** The template had the identical rebase bug
(its issue #67) and now rebases while clean *and* ships a regression test. Our copy
had drifted and kept the broken ordering — which is the whole of defect #1. No
upstream report needed; we adopted the upstream approach.

**Still broken upstream → reported (§6).** The `local != remote` mislabel
(template `release.yml:735-736`) and the tag-before-push-rebase hazard (template
`release.yml:872` vs `:879-903`) both still exist upstream.

**Other drift found, recorded for follow-up.** These are pre-existing gaps rather
than causes of the analysed failures, and restoring them is a larger change than
this PR should carry:

| Template feature | State here | Risk |
|---|---|---|
| Cargo.lock guard job + `scripts/check-cargo-lock.rs` | deleted | **False negative** — cache keys use `hashFiles('**/Cargo.lock')`, so a stale lock silently poisons cache keys |
| `smoke-test-published-crate` | removed | A broken publish is not caught |
| `setup-buildx-resilient` composite action on the GHCR path | removed | Transient buildx setup failures fail the release |
| Test matrix across OSes | narrowed to `ubuntu-latest` | Platform regressions escape |

---

## 6. Upstream reports (R7)

| Report | Defect | Why it belongs there |
|---|---|---|
| [rust-…-template#94](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/94) | Tag created before the push-retry rebase → can point at an orphaned commit | Still present in the template's `scripts/version-and-commit.rs:865-905`; every repo generated from it inherits it |
| [rust-…-template#95](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/95) | `local != remote` mislabelled as "behind" | Still present at `scripts/version-and-commit.rs:731-743`. This message is what initially misdirected our own investigation |
| [lindera#750](https://github.com/lindera/lindera/issues/750) | **Two** bugs in `lindera-dictionary/src/assets.rs`'s `DOCS_RS` branch: the dummy dictionary ignores `src_subdir` (fails on a *fresh* sandbox), and `fs::create_dir` is not idempotent (fails on re-run) | The defect and its fix live there; it affects **every** crate depending on lindera, not just ours |
| [meta-language#181](https://github.com/link-foundation/meta-language/issues/181) | Depends on `lindera` non-optionally, so every dependent's docs.rs build fails | `meta-language` itself fails on docs.rs (build 3577282). A lindera 4.x bump does **not** fix this (§4.3), so the ask is to feature-gate `lindera` so dependents can opt out |

Each report includes a reproducible example, a workaround, and a concrete fix
suggestion, as the issue requires.

Filed in **this** repository rather than upstream, for the one defect that is a
design decision rather than a patch:

| Report | Defect | Why separate |
|---|---|---|
| [#738](https://github.com/link-assistant/formal-ai/issues/738) | Every release leaves `fragment-release-map.tsv` stale → `main` goes red on the next unrelated PR | Unlike the `CHANGELOG.md` half of the same check (§4.7, fixed here), this **cannot** be fixed inside the release commit: the map's third column is the release commit's own SHA. Fixing it means dropping the SHA column or amending post-commit — a design choice, and the amend route would touch the push/rebase path repaired in §4.1 |

---

## 7. Known components / libraries considered

| Problem | Existing component | Verdict |
|---|---|---|
| Runner disk exhaustion | [`jlumbroso/free-disk-space`](https://github.com/jlumbroso/free-disk-space), [`easimon/maximize-build-space`](https://github.com/easimon/maximize-build-space) | **Not adopted.** Both are third-party actions in the release path — a supply-chain surface on the job that publishes to crates.io and GHCR. Our need is four `rm -rf` paths and a `df`; `scripts/free-runner-disk.sh` is ~40 lines, has no external dependency, and is testable locally. Their approach (remove unused SDKs) is what we implement. |
| Concurrent-release races | GitHub `concurrency` groups | **Adopted** for `desktop-release.yml` (§4.4). Not sufficient for `release.yml`: the race there is between *separate pushes to main*, each of which legitimately must release, so serialising is not the answer — rebasing correctly is. |
| Release automation | `release-plz`, `cargo-release`, `semantic-release` | **Not adopted here.** Any of them would replace `version-and-commit.rs` wholesale, and `release-plz` handles this exact race properly. That is a strategically sound migration but far beyond a CI-reliability fix; recorded as a follow-up candidate. |
| Build provenance | `actions/attest` | **Already in use**; the defect was ordering (§4.4), not the component. |
| docs.rs build isolation | `cargo-docs-rs` / `DOCS_RS` env guard | Would let `lindera` detect the docs.rs sandbox — this is the suggested fix in the upstream report, not something we can apply from here. |

---

## 8. Data index

| File | What it proves |
|---|---|
| [`data/issue-736.json`](./data/issue-736.json) | The issue as filed, for the requirement list in §3 |
| [`data/run-2921*.json`, `run-2931*.json`, `run-2948*.json`](./data) | The four failing runs: timestamps, commits, `conclusion: failure` |
| [`data/jobs-*.json`](./data) | `Auto Release` is the only non-success job in every one of them |
| [`data/job-87580040366.json`](./data/job-87580040366.json), [`data/job-86709184220.json`](./data/job-86709184220.json) | Step 8 `Collect changelog and bump version` = failure → defect #1 |
| [`data/job-87583325043.json`](./data/job-87583325043.json), [`data/job-87020644690.json`](./data/job-87020644690.json) | **No failed step at all** → the runner itself died → defect #2 |
| [`data/annotations-*.json`](./data) | The `No space left on device` annotations — the only surviving evidence for defect #2 |
| [`data/autorelease-87580040366.log`](./data/autorelease-87580040366.log) | The verbatim `cannot rebase` failure |
| [`data/api-404-job-87583325043.txt`](./data/api-404-job-87583325043.txt) | The log blob 404 — itself a symptom of defect #2 |
| [`data/docsrs-build-3868612-formal-ai-0.296.0.log`](./data/docsrs-build-3868612-formal-ai-0.296.0.log) | The `lindera-jieba` `File exists (os error 17)` failure → defect #3 |

The two full-pipeline logs for runs 29484631709 (3.5 MB) and 29485000765 (2.3 MB)
are not committed; everything they contain about the failures is in the
`Auto Release` extract above. They can be regenerated with
`gh run view <id> --repo link-assistant/formal-ai --log`.
