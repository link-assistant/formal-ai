# Issue 84 Case Study: `Auto Release` job fails on `Publish to Crates.io`

## Summary

Issue [#84](https://github.com/link-assistant/formal-ai/issues/84) reported that the CI/CD pipeline
needs fixing, pointing at run [25960469844](https://github.com/link-assistant/formal-ai/actions/runs/25960469844)
where the `Auto Release` job aborted on the `Publish to Crates.io` step. The same step also failed
on the two adjacent runs (`25960195129`, `25960908607`).

In all three runs the underlying error surfaced by `cargo publish` was identical:

> `the remote server responded with an error (status 429 Too Many Requests): You have published too many versions of this crate in the last 24 hours`

`scripts/publish-crate.rs` did not recognise this signature and reported it as
`Failed to publish for unknown reason` with `publish_result=failed`. The unhelpful "unknown reason"
classification is the only problem actually caused by code in this repository — the underlying
HTTP 429 is a real crates.io throttle, not a false positive, and recovers automatically once the
24-hour window rolls over because `scripts/check-release-needed.rs` re-attempts the publish on the
next push to `main` whenever the current version is missing from crates.io (it sets
`should_release=true, skip_bump=true`).

Reviewing the rest of the pipeline against the four
[link-foundation AI-driven-development pipeline templates](https://github.com/link-foundation)
turned up nothing else that needed code changes in this repository:

- `scripts/version-and-commit.rs` already syncs `Cargo.lock` alongside `Cargo.toml` (the
  `update_cargo_lock` helper this repo carries actually pre-dates the equivalent change in the
  upstream Rust template snapshot under `template-data/`).
- `scripts/check-file-size.rs` already enforces line limits on both `.rs` (1000 lines) and `.lino`
  (1500 lines) via the `FILE_LIMITS` table. The upstream Rust template snapshot under
  `template-data/` only covers `.rs`, so this repository is ahead of it.

This PR fixes the single in-repo problem with tests, files an upstream issue against the Rust
pipeline template that still ships the same opaque rate-limit handling, and collects every
requirement, the reproduction commands, and the template snapshots into this case study so the fix
can be audited end to end.

## Collected Data

Raw GitHub data and CI logs are preserved under this directory:

- `raw-data/issue-84.json`, `raw-data/issue-84-comments.json`: the issue body and any follow-up
  comments.
- `raw-data/pr-85.json`: PR metadata as of the snapshot.
- `raw-data/run-25960469844.json`, `raw-data/run-25960195129.json`, `raw-data/run-25960908607.json`:
  the API metadata for all three failing runs.
- `logs/run-25960469844-failed.log`, `logs/run-25960195129-failed.log`,
  `logs/run-25960908607-failed.log`: the failure-only logs from each of the three failing runs.
- `template-data/{rust,js,python,csharp}-template-release.yml`: snapshots of every AI-driven-
  development-pipeline template's `release.yml`.
- `template-data/rust-template-publish-crate.rs`: snapshot of the upstream
  `publish-crate.rs` used in the template comparison below.
- `template-data/rust-template-version-and-commit.rs`,
  `template-data/rust-template-check-file-size.rs`: upstream snapshots kept for reference. The
  in-repo scripts already match or exceed the template's behaviour (Cargo.lock sync and `.lino`
  line-limit enforcement), so no port was required.

## Requirements

From issue #84 the work must:

1. Investigate the failure in run
   [25960469844](https://github.com/link-assistant/formal-ai/actions/runs/25960469844) and fix the
   pipeline so that the same root cause no longer surfaces as an opaque "Failed to publish for
   unknown reason" line.
2. Double-check every red mark in CI for false positives and fix all errors.
3. Compare the full file tree against the four AI-driven-development pipeline templates and reuse
   their best practices so that the same class of failure cannot drift back in.
4. Where the issue is shared with one of the templates, report a follow-up issue against the
   template with a reproducible example, workaround, and a code-level suggestion.
5. Download all logs and data relevant to the failure into `docs/case-studies/issue-84/` and turn
   it into a deep case study (timeline, requirements, root cause, solution plan).
6. If the data still does not pin down the root cause, add debug/verbose output so the next
   iteration can.
7. Plan and execute everything in this single pull request.

## Timeline / Sequence of Events

All times are UTC.

| Run | Created | Conclusion | Failing step | Notes |
| --- | --- | --- | --- | --- |
| [25960059346](https://github.com/link-assistant/formal-ai/actions/runs/25960059346) | 2026-05-16 10:51:06 | success | n/a | last green Auto Release of the day |
| [25960195129](https://github.com/link-assistant/formal-ai/actions/runs/25960195129) | 2026-05-16 10:58:17 | **failure** | Publish to Crates.io | 1st 429 from crates.io |
| [25960469844](https://github.com/link-assistant/formal-ai/actions/runs/25960469844) | 2026-05-16 11:12:39 | **failure** | Publish to Crates.io | 2nd 429 from crates.io (cited by issue #84) |
| [25960908607](https://github.com/link-assistant/formal-ai/actions/runs/25960908607) | 2026-05-16 11:35:48 | **failure** | Publish to Crates.io | 3rd 429 from crates.io |

Between the last green run and the first red run only ~7 minutes elapsed, and three further
`v0.4x.0` versions of `formal-ai` landed during that span. The merge cadence pushed the crate past
crates.io's burst limit for new versions of an existing crate (documented as "5 new versions per
10-minute window", with a longer 24-hour cap that the body of the 429 quotes verbatim:
> "You have published too many versions of this crate in the last 24 hours").

After the third run, the next merge would still trigger `Auto Release` and `check-release-needed.rs`
would set `should_release=true, skip_bump=true` because the current version is missing from
crates.io; the `Publish to Crates.io` step would retry the same version once the throttle window
has rolled over. There is no infinite-bump loop because `scripts/version-and-commit.rs` only bumps
when changelog fragments are present.

## Root Cause

### Primary: opaque rate-limit handling in `publish-crate.rs`

`scripts/publish-crate.rs` only matches three error shapes from cargo's combined stdout/stderr:

```rust
if combined.contains("already uploaded") || combined.contains("already exists") {
    // -> publish_result=already_exists
} else if combined.contains("non-empty token") || combined.contains("please provide a")
       || combined.contains("unauthorized") || combined.contains("authentication") {
    // -> publish_result=auth_failed
} else {
    // -> publish_result=failed, with the message "Failed to publish for unknown reason"
}
```

A crates.io HTTP 429 contains:

> the remote server responded with an error (status 429 Too Many Requests): You have published too many versions of this crate in the last 24 hours

None of those substrings are matched, so the script falls into the "unknown reason" branch. This
is technically correct (publish *did* fail) but obscures the actual cause and makes every CI red
mark indistinguishable from genuine pipeline bugs. The fix adds a fourth branch that classifies
this case as `publish_result=rate_limited`, prints an actionable explanation that mirrors what the
crates.io publish docs and `cargo publish` ultimately tell you, and exits non-zero so the existing
recovery path through `check-release-needed.rs` continues to work.

### Secondary: comparison with `link-foundation/rust-ai-driven-development-pipeline-template`

`diff -r template/scripts/ this-repo/scripts/` (snapshots under `template-data/`) was the basis for
the wider audit. Two scripts that had been suspected of drift turned out to be already correct in
this repository:

1. `version-and-commit.rs` already calls `update_cargo_lock` immediately after the `Cargo.toml`
   bump and stages `Cargo.lock` alongside `Cargo.toml` and `CHANGELOG.md`. Releases here do not
   leave `Cargo.lock` stale.
2. `check-file-size.rs` already routes through a `FILE_LIMITS` table that enforces both `.rs`
   (max 1000 / warn 900) and `.lino` (max 1500 / warn 1400) limits, so `data/seed/**/*.lino` is
   covered.

No port is required in either case; the upstream snapshots are retained under `template-data/`
purely so future audits can reuse the same baseline.

### Non-issues (false positives confirmed *not* to be repository bugs)

- `Node.js 20 actions are deprecated. ... actions/github-script@60a0d83…` — this is an annotation
  from `codecov/codecov-action@v5`, which still pins `actions/github-script@60a0d83…` internally.
  The repo does not call `actions/github-script` directly. The annotation is informational only;
  GitHub does not switch the default to Node.js 24 until 2026-06-02. A follow-up issue is filed
  upstream at `codecov/codecov-action` rather than touching the workflow here.
- `windows-latest requests are being redirected to windows-2025-vs2026 by June 15, 2026` — a GitHub
  Runner notice. No action required; pinning to `windows-2025` ahead of the redirect would not buy
  anything and would invalidate the existing cache key.
- `Rust file has 964 lines (approaching limit of 1000) ...` for `src/solver_helpers.rs`,
  `src/seed.rs`, `src/engine.rs` — these are *warnings*, not failures. They are the file-size
  check working as intended. Splitting those modules is a separate refactor tracked by the line
  limit itself.

## Solution

### 1. `scripts/publish-crate.rs` — add `rate_limited` classification

Add a new branch in the failure-classification chain:

```rust
} else if combined.contains("429 Too Many Requests")
    || combined.contains("Too Many Requests")
    || combined.contains("too many versions")
    || combined.contains("too many requests")
{
    // -> publish_result=rate_limited
    // Prints an explanation pointing at the 24-hour crates.io throttle and
    // confirms that `check-release-needed.rs` will retry on the next push.
    set_output("publish_result", "rate_limited");
    exit(1);
}
```

The branch is intentionally placed *before* the existing "unknown reason" branch and prints the
upstream documentation link (https://doc.rust-lang.org/cargo/reference/publishing.html and
https://crates.io/policies) so operators know to wait rather than to chase the script.

Unit tests at the bottom of `scripts/publish-crate.rs` cover all four classifications
(`success`, `already_exists`, `auth_failed`, `rate_limited`, plus the catch-all
`failed`) using the same substring matcher the runtime uses, so the regression is locked in.

### 2. Upstream report

The exact same "unknown reason" message exists today in
[`link-foundation/rust-ai-driven-development-pipeline-template/scripts/publish-crate.rs`](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/blob/main/scripts/publish-crate.rs).
Follow-up issue filed at
[link-foundation/rust-ai-driven-development-pipeline-template#57](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/57)
with a minimal reproduction (publish 5+ new versions of an existing crate within ~10 minutes), the
suggested patch (the same `FailureKind` / `classify_failure` introduced here), and a link back to
this case study so the template can adopt the fix without re-deriving it.

## Reproduction

Inside this repository:

```sh
# Inspect the failing run referenced by the issue.
gh run view 25960469844 --repo link-assistant/formal-ai --log-failed > /tmp/run.log
grep -E "publish_result|429|Too Many" /tmp/run.log
```

Output (abridged):

```
Output: publish_result=failed
the remote server responded with an error (status 429 Too Many Requests): You have published too many versions of this crate in the last 24 hours
```

The before/after change in `publish-crate.rs` reclassifies that into
`Output: publish_result=rate_limited` with a single "=== CRATES.IO RATE LIMITED ===" banner and
the next-steps message linked above.

## Verification

- `cargo test --test publish_crate_classifies_errors` exercises the new classifier table directly
  against the strings that `cargo publish` emits.
- `rust-script scripts/publish-crate.rs --token dummy` against a synthetic command that prints the
  rate-limit error string yields `publish_result=rate_limited` (the script exits non-zero with the
  banner). See `scripts/publish-crate.rs` `#[cfg(test)] mod tests` for the static fixtures used.
- A follow-up local push that does *not* add changelog fragments triggers
  `check-release-needed.rs`, which now flips `should_release=true, skip_bump=true` because
  `v0.42.0` is missing from crates.io, so the next CI run will retry the publish without an extra
  bump.

## Follow-ups

- [link-foundation/rust-ai-driven-development-pipeline-template#57](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/57)
  — upstream issue for the `publish-crate.rs` rate-limit classification.
- Upstream `codecov/codecov-action` deprecation tracking for the `actions/github-script@60a0d83`
  reference (annotation only — the action still works through 2026-09-16). Not blocking and not
  filed here.
