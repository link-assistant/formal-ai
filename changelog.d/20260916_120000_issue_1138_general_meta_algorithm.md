---
bump: minor
---

### Added

- Issue #1138: the plan set and implementation of a general, self-coding
  meta algorithm. One `Need` record connects every step; the universal loop
  performs live concept lookup over the trusted sources registry instead of
  logging that it cannot fetch; a content-addressed sense ledger forgets and
  rediscovers; every obligation needs an execution `Evidence` record before
  it is satisfied; prerequisites such as a missing compiler become needs
  solved through trusted publishers with workspace-scoped installs; one
  repository workspace protocol serves SWE-bench, the coding ladder and
  self-coding with a seed-driven default-deny command allowlist; capability
  routing is decided by object type, act and locus from seed with a 420-case
  held-out suite in five languages; selection heuristics (least action, TRIZ
  contradictions, refutation-first search, balanced splitting) are registry
  methods; learned methods must change a held-out answer before adoption.

### Fixed

- Prose could reach `/bin/sh -c` on the agent path and a non-zero exit could
  be reported as completed; the seed allowlist and two-valued command
  outcomes close both.
- GitHub work items are read as structured source through `gh` when a real
  shell is available; fetch-only clients receive a data-declared extraction
  prompt instead of the solve request. This prevents model-backed fetch tools
  from recursively solving a task and returning generated code as issue text,
  while preserving a bounded fetch fallback when the CLI read is unavailable.
- The closure audit no longer counts its own generator's output; the sixteen
  generated closure shards are deleted and the debt ratchet is strict in
  both directions.
- The tests-as-documentation gate parses Rust assertions structurally, so an
  answer mentioned only in diagnostic formatting no longer masquerades as an
  exact behavioral contract.
- Browser synchronous-handler membership, invocation metadata and precedence
  now come from a registered Links Notation seed instead of a worker-local
  array. A web-stage gate rejects duplicate inventories, missing bindings,
  stale generated seed lists and fixed-path debt scans that overlook the real
  dispatcher.
- Repository completion now compares evidence-backed current facts with an
  independently formalized goal state. Missing, unevidenced and mismatched
  requirements remain typed Need/obligation gaps, so mergeability, a diff or a
  partial green check cannot by itself declare a repository task complete.
- Selection is now a registry capability: source-linked TRIZ contradictions
  are exercised by a 20-task corpus in five languages, refutation-first search
  precedes sampling, binary splitting reports underivable tasks honestly, and
  approach deduplication retains the first historical source.
- Release publication is now convergent and observable: crates.io throttling
  fails the job instead of yielding a green partial release, automatic reruns
  resume the prepared current-main version without double-bumping or
  double-tagging, publication state is rechecked after synchronization, and
  Cargo verifies the generated `.crate` before any downstream artifact is
  published.
