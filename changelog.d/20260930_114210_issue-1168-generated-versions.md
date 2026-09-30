bump: minor
---

### Added

- Generated CI workflows now resolve every third-party pin at generation
  time instead of shipping memorized strings (issue #1168): the action refs
  (`actions/checkout`, `actions/setup-java`, `fwilhe2/setup-kotlin`,
  `actions/setup-python`), the Kotlin and CPython versions, and the Java LTS
  come from the publishers' own APIs through the existing
  `CachedSourceClient`, are recorded in the derivation as `SourceCapture`s,
  and degrade loudly — cache first, then the shipped
  `data/seed/toolchains.lino` baseline measured 2026-09-30 — with the origin
  written as a comment on the generated workflow. A new gate
  (`scripts/check-generated-version-literals.rs`) fails on any literal
  action SHA or bare toolchain version under `data/`.
