bump: patch
---

### Fixed

- The projection rebuild-cost ratio pin (`issue_1106_projection_reuse`) no
  longer flakes on loaded CI runners: each leg (append and rebuild) is now the
  minimum of three sampled timings instead of a single wall-clock sample, so
  one contended sample cannot speak for the leg. Scheduling noise only ever
  adds time, so the minimum estimates the intrinsic cost, and the fsync-per-
  doublet regime the pin guards against (issue #710, PR #888) stays two orders
  of magnitude past the bound under any sample count. The failure message now
  names the binding budget explicitly. Main run 36263664670 failed the merge of
  PR #1144 with a 2.27 s rebuild against the 2 s generosity floor — 13 % over
  on a runner loaded by the rest of the suite — while the identical commit had
  passed the same test on its branch run hours earlier.
