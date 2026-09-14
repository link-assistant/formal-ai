# Online research — facts checked against primary sources

Task requirement T3: *"search online for additional facts and data … Also check
online for known existing components/libraries that solve a similar problem or
can help."*

Everything below was fetched during this investigation rather than recalled.
Each claim carries the URL it came from and, where the wording decides a design
question, the vendor's own sentence. Claims that the collected logs do **not**
support are marked as such — a documented risk is not the same as an observed
defect, and this document keeps the two apart.

---

## 1. GitHub Actions cache: quota, eviction and ref scoping

Source: <https://docs.github.com/en/actions/reference/dependency-caching-reference>

| Fact | GitHub's wording |
| --- | --- |
| Quota | "By default, the limit is 10 GB per repository, but this limit can be increased by enterprise owners, organization owners, or repository administrators." |
| Eviction | "Once a repository has reached its maximum cache storage, the cache eviction policy will create space by deleting the caches in order of last access date, from oldest to most recent." |
| Retention | "GitHub will remove any cache entries that have not been accessed in over 7 days." |
| Ref scoping | "Workflow runs can restore caches created in either the current branch or the default branch (usually `main`)." … "Workflow runs cannot restore caches created for child branches or sibling branches." |

Three consequences for this repository, each of which is measured in README §5
rather than assumed:

1. **The quota is shared and the policy is LRU by *last access*, not by size.**
   A namespace that writes many entries and reads few of them therefore evicts
   the namespaces that *are* read. That is the mechanism behind D2: 48
   `buildkit-blob-*` entries (4.91 GB, 42.9% of the quota) are written by every
   image build and read only by the next image build, while the six
   `*-cargo-*` entries (0.99 GB) are read by nearly every job.
2. **The measured total is 11.44 GB against a 10 GB default quota.** The
   repository is already past the eviction threshold, so every run is evicting
   something.
3. **Ref scoping makes 1,731 entries (31%) permanently unreadable.** They belong
   to `refs/pull/1074/merge`; GitHub's rule above means `main` can never restore
   them, yet they occupy the shared quota until the 7-day rule removes them.
   This is not a misconfiguration — it is the documented behaviour — but it is
   why "how much quota does `main` actually have" is much smaller than 10 GB.

### 1.1 Policy changes since the quota was designed

* <https://github.blog/changelog/2025-11-20-github-actions-cache-size-can-now-exceed-10-gb-per-repository/>
  introduces two admin-settable policies, a **cache size eviction limit (GB)**
  and a **cache retention limit (days)**. The defaults are unchanged — "you
  receive a 10 GB cache size limit and a seven-day retention limit at no
  additional cost" — and raising the size limit is billable: "If you increase
  the cache size limit beyond your plan's defaults, you will be charged for
  extra cached storage."

  **Bearing on this issue:** raising the limit is an available lever, but it is
  a paid one that treats the symptom. D2's fix (scoping the image cache so
  builds stop overwriting one another, and stopping the eight-way duplication
  of the cargo registry) reduces demand instead, and costs nothing. Recorded
  here so the trade-off is explicit rather than implicitly foreclosed.

* <https://github.blog/changelog/2025-09-29-new-date-for-enforcement-of-cache-eviction-policy/>
  is the enforcement-date announcement for the eviction policy above.

### 1.2 A documented limit this repository *has* hit — and cannot see

GitHub staff announced a rate limit of **"200 cache entry uploads per minute per
repository"**, with a `429` response carrying `Retry-After` when exceeded
(<https://github.com/nodejs/node/issues/61436>, January 2026; the same notice
was cross-posted to `NixOS/nix#15016` and `apache/opendal#7150`).

This repository holds **5,439 `sccache/*` entries** — sccache's GHA backend
writes one cache entry per compilation object — so the shape that triggers the
limit is present.

**Correction.** An earlier draft of this file recorded the limit as a risk that
"has not triggered". That was wrong, and wrong for an instructive reason: the
grep behind it filtered out the very lines it was looking for. It has triggered
at least twice in the collected evidence, on both halves of the cache round
trip.

**On restore** — `ci-logs/coverage-history/job-97202496724-run-32642782572.log`,
lines 548-550, 2026-08-23:

```
##[warning]You've hit a rate limit, your rate limit will reset in 6 seconds
##[warning]Failed to restore: Failed to GetCacheEntryDownloadURL: Rate limited: Failed request: (429) Too Many Requests: rate limit exceeded
Cache not found for input keys: Linux-cargo-coverage-a75c0f55…, Linux-cargo-coverage-
```

The third line is the finding. `Cache not found for input keys` is *the same
line a genuine miss prints*. A refused request and an absent entry are
indistinguishable in the log unless the reader happens to look one line up — and
a job that then recompiles every dependency looks, from its duration alone,
exactly like a cold cache. Coverage runs in this window were being diagnosed as
cache misses; at least one of them was a rate-limited request instead.

**On save** — `ci-logs/main-head-701d6a45/run-33955786067.log`, lines 1533-1534,
2026-09-05, in the `Post Cache cargo registry` step of the run at `main` head:

```
##[warning]You've hit a rate limit, your rate limit will reset in 1 seconds
Failed to save: Unable to reserve cache with key Linux-cargo-cli-matrix-7071513d…, another job may be creating this cache.
```

Here the mis-attribution is in the message itself: `actions/cache` reports the
refused reservation as **"another job may be creating this cache"** — a
concurrency race — when the warning immediately above says it was a rate limit.
Anyone reading only the second line would go looking for a duplicate job that
does not exist.

**Bearing on this issue.** Three consequences, all of them in scope:

1. These are `##[warning]` annotations on runs that report green. They are
   literally "warnings in CI/CD" of the kind issue #1076 asks to account for,
   and they were invisible because nothing summarises cache outcomes.
2. It makes D3 a *defect* rather than an ergonomic nicety. The cache-outcome
   summary step added to `.github/actions/cache-cargo-registry/action.yml`
   reports hit / prefix-restore / miss per invocation, and under
   `FORMAL_AI_CI_VERBOSE` prints a warning on a miss that names the rate-limit
   possibility explicitly, so the next occurrence is attributable from the
   summary instead of requiring a full log download.
3. It is a second independent reason to prefer *reducing* the number of cache
   writers (D2) over raising the quota: the 10 GB cap is billable to lift, and
   the 200/min limit is not liftable at all.

## 2. buildx `type=gha`: `scope` and `mode`

Source: <https://docs.docker.com/build/cache/backends/gha/>

The parameter table gives, verbatim:

| Parameter | Where | Type | Default |
| --- | --- | --- | --- |
| `scope` | `cache-to`, `cache-from` | String | **`buildkit`** |
| `mode` | `cache-to` | `min`,`max` | `min` |

and the warning that decides D2's fix:

> "If you build multiple images, each build will overwrite the cache of the
> previous, leaving only the final cache."

Two design decisions follow, and they point in *opposite* directions — which is
why an earlier draft of this work got one of them wrong:

* **`scope` must be set.** Every `cache-to`/`cache-from` in this repository
  previously used the default scope `buildkit`, so all image builds shared one
  scope and overwrote each other. Fixed at all five sites with
  `scope=docker-image`.
* **`mode=max` must stay.** An earlier draft changed `mode=max` to `mode=min`
  to shrink the 4.91 GB. That is wrong: `mode=min` exports only the *final*
  stage's layers, and this image is multi-stage, so the expensive Rust compile
  layers would stop being cached entirely — reintroducing the uncached release
  build that issue #977 was filed for (run 31065367736). The change was
  reverted and the test that demanded it was corrected; see
  `tests/unit/ci-cd/issue_1076.rs::container_build_caches_are_bounded_and_scoped`,
  whose doc comment records the mistake so it is not repeated.

  The bound on Docker cache growth is therefore *the number of exporting
  builds*, not the export mode. The test enforces at most two exporters (the
  two from-source GHCR publishes required by issue #1057).

## 3. Existing components that solve these problems

T3 asks explicitly for prior art. Each row below was checked against the
project's own documentation, and the last column states why it was or was not
adopted — a component that is right for someone else's pipeline is not
automatically right for this one.

| Problem | Existing component | Verified capability | Decision here |
| --- | --- | --- | --- |
| A step overruns and GitHub reports `cancelled` instead of `failure` (D1) | GNU coreutils `timeout(1)` | `--kill-after` escalates SIGTERM→SIGKILL; exits `124` on timeout | Already the model for `scripts/run-with-budget-warning.sh`, which adds the 70%-of-budget warning and the `::error` annotation that turns the overrun red. Kept. |
| Per-test (not per-job) timeouts | `cargo-nextest` `slow-timeout` | `period` marks a test slow; `terminate-after = N` terminates after N periods; `grace-period` (default 10s) before SIGKILL; a timed-out test reports `TIMEOUT` and "tests that time out are treated as failures" (<https://nexte.st/docs/features/slow-tests/>) | **Gap identified.** nextest is already installed in this repository (`.github/workflows/macos-core-tests.yml:62`) but there is **no `.config/nextest.toml`**, so `slow-timeout`/`terminate-after` are unset and a hung test still consumes the whole job cap. The coverage job runs `cargo llvm-cov` over `cargo test`, not nextest, so this is not a drop-in fix — recorded in `solution-plans.md` as the structural option for D11. |
| No runner telemetry, so a 7.4x slowdown cannot be attributed (D11) | `catchpoint/workflow-telemetry-action@v2` | Collects "CPU Load (user and system) in percentage", "Memory usage (used and free) in MB", "Network I/O (read and write) in MB", "Disk I/O (read and write) in MB"; publishes to the job summary and (by default) a PR comment | **Not adopted.** It defaults to commenting on every pull request and needs `pull-requests: write` — an always-on, write-scoped third-party action to diagnose an intermittent condition. `scripts/report-runner-capacity.sh` collects the same class of data (`nproc`, load average, `/proc/stat` steal, `MemAvailable`, `df`), is off unless `FORMAL_AI_CI_VERBOSE` is set, needs no token, and is auditable in-tree. Recorded as the fallback if the in-tree script proves insufficient. |
| Workflow *syntax* errors | `rhysd/actionlint` | Delegates `run:` bodies to ShellCheck; **silently exits 0 for shell checks when ShellCheck is absent from PATH** | Already used; the container image `docker://rhysd/actionlint` bundles ShellCheck and pyflakes, which is what all four templates use and what this repository now matches. |
| Workflow *security* defects (D10) | `zizmor` / `zizmorcore/zizmor-action` | Detects `template-injection`, `unpinned-uses`, `dangerous-triggers`, `artipacked`, `self-repository` | **Adopted**, matching all four templates: `.github/workflows/workflows.yml` + `.github/zizmor.yml`. |
| Cache quota hygiene | `gh cache list` / `gh cache delete`; `MyAlbum/purge-cache` | Enumerates and deletes entries by key/ref | Used as the *measurement* tool (`analysis/actions-caches-fresh.tsv`, 5,497 entries). Not adopted as a scheduled job: deleting entries a run may still need converts a quota problem into a cache-miss problem. The fix reduces what is written instead. |
| Runner disk exhaustion | `jlumbroso/free-disk-space` | Removes preinstalled toolchains to reclaim ~30 GB | Equivalent already in place inline ("Free up runner disk space", `release.yml:331`), ordered before the multi-gigabyte cache restore — an ordering pinned by `tests/unit/ci-cd/workflow_release.rs`. |

## 4. What the research did *not* settle

Stated so the limits of this document are visible:

* **The 7.4x coverage slowdown (D11) has no external explanation.** Nothing in
  GitHub's status history, the runner image changelog (`ubuntu-24.04`,
  provisioner `20260828.587`, runner `2.337.0` — identical between the fast and
  the slow run) or the cache documentation accounts for identical tests taking
  7.4x longer. This is precisely why the fix for D11 is *instrumentation*, not
  a code change: the data needed to attribute it was never collected. The
  off-by-default verbose mode exists so the next occurrence is diagnosable.
* **Whether the 200-uploads/minute limit is being approached** cannot be
  answered from the logs, because sccache does not report its upload count. It
  is a risk, not a measurement.
