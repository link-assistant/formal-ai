# D9 -- half of every compiler-cache write is refused, and nothing says so

## What was observed

`sccache --show-stats` prints `Cache write errors` as one row among twenty.
Nothing reads that row, nothing compares it to a threshold, and no annotation
is ever produced from it. A run whose compiler cache accepted half of what it
was offered is therefore indistinguishable, in the Actions UI, from a healthy
one.

## Measurement

Source: the eleven `--stats-format=json` dumps recoverable from the archived
logs of run **34095902681** (`main`, `6039c4d9`, 2026-09-07). Extracted by the
inline script recorded in `sccache-write-errors.txt`; the two rows with zero
write attempts are the pre-compilation `--show-stats` calls and are excluded
from the table below.

| first seen | job | writes | write errors | refused |
|---|---|---:|---:|---:|
| 07:35:57 | Summarization quality ratchet (issue #895) | 13 | 137 | 91.3% |
| 07:38:38 | Task Ladder (issue #840 dataset) | 3 | 126 | 97.7% |
| 07:39:06 | Write-Effect Ladder (issue #916 rungs) | 44 | 111 | 71.6% |
| 07:39:21 | Question necessity ratchet (issue #920) | 176 | 89 | 33.6% |
| 07:40:37 | Build binaries and tests | 62 | 30 | 32.6% |
| 07:42:00 | Lint and Format Check | 377 | 374 | 49.8% |
| 07:52:35 | macOS Core Tests / Build macOS test archive | 36 | 1 | 2.7% |
| 08:06:35 | Test (macos-15-intel / specification) | 66 | 0 | 0.0% |
| | **total** | **843** | **868** | **50.7%** |

`cache_read_errors` is 0 everywhere, and `compile_fails` is a constant 6 on
every Linux job (9 on the lint job, 1 on coverage) -- the shape of build-script
probe compilations that are *expected* to fail, so it is recorded here and not
treated as a defect.

Note that the human-readable table's `Cache errors` row reads `0` in all of
these runs: it renders `cache_errors`, a different counter from
`cache_write_errors`. The row that reads 137 is further down and easy to skim
past.

## Why it matters

A refused write is a compiled artifact that never entered the shared cache, so
the next run compiles that crate again. This repository has arrived at compile
time from the other direction three times -- issues #1017, #1021, and #1081's
own D1 (`Test (macos-15-intel / specification)` killed at its 1400s budget) --
and a cache that stores half of what it produces is the cheapest available
explanation for the pressure.

## Root cause: two candidates, and what separates them

**Candidate 1 -- rate limiting by the GitHub Actions Cache service.** Documented
upstream in [Mozilla-Actions/sccache-action#50](https://github.com/Mozilla-Actions/sccache-action/issues/50),
where the backend answers `Request was blocked due to exceeding usage of
resource 'Count' in namespace ''` and sccache counts the result under exactly
this counter.

**Candidate 2 -- concurrent writers of identical keys.** The five Ubuntu jobs in
the table overlap between 07:35 and 07:42 and compile the same
`x86_64-unknown-linux-gnu` crates against one cache namespace
(`ghac, name: 90541733...`). The two macOS jobs run alone, at 07:52 and 08:06,
in a key space nobody else is writing, and refuse ~0%.

Both candidates predict an error rate that rises with *simultaneity*, which is
what the table shows. One variant is already refuted: the rate does **not** rise
with finishing order -- the two earliest finishers are the worst two rows and
the two latest are the best two -- so "another job got there first" in the
sequential sense does not explain it.

What separates the two candidates is the backend's own HTTP response, which
sccache emits only under `SCCACHE_LOG=debug`.

## What was changed

1. `scripts/check-sccache-write-health.sh` -- computes the refused share from
   `--show-stats --stats-format=json` and emits a `::warning` at or above
   `SCCACHE_WRITE_ERROR_WARN_PERCENT` (default 25), staying silent below
   `SCCACHE_WRITE_MIN_ATTEMPTS` (default 20) attempts, where the ratio is noise
   rather than a measurement. It always exits 0: a diagnostic that fails a step
   replaces the finding with a worse one.
2. `scripts/run-with-budget-warning.sh` calls it once after the wrapped command
   finishes, whatever the outcome. That covers every budgeted Rust step in every
   workflow -- which is exactly the set of steps the refused writes make slower
   -- without touching a single workflow file.
3. `.github/actions/setup-sccache/action.yml` -- the opt-in diagnostics step now
   runs **before** the server is started, and also sets `SCCACHE_ERROR_LOG`.

Item 3 is a defect in its own right. The verbose mode already existed, but it
wrote `SCCACHE_LOG=debug` to `$GITHUB_ENV` *after* `sccache --start-server`.
sccache's logging belongs to the server process -- "You can set the
`SCCACHE_ERROR_LOG` environment variable to a path and set `SCCACHE_LOG` to get
the server process to redirect its logging there" -- and the daemon reads its
environment once, at spawn; the configuration reference says the same from the
other side, "Note that some env variables may need sccache server restart to
take effect." Turning the flag on therefore produced no debug output in the job
you turned it on for. That is why the root cause above is still two candidates:
the switch that would have decided it was wired to do nothing.

Setting the `FORMAL_AI_CI_VERBOSE` repository variable to `true` and rerunning
now collects the discriminating evidence.
