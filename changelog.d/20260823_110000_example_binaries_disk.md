---
bump: patch
---

### Fixed

- Stop linking 116 example binaries on every Rust commit. The pre-commit hook
  ran `cargo clippy --all-targets`, which links every example into a ~190MB
  binary — and cargo keeps both a hashed and an unhashed copy, so a single run
  left about 27GB in `target/debug/examples`. The `run_clippy` CI gate already
  used the cheaper split, `cargo clippy --lib --bins --tests` plus
  `cargo check --examples`, which type-checks examples without linking them; the
  hook now matches it, and a test keeps the two from drifting apart.

- Remove linked example binaries when pruning. `cargo sweep` reasons about what
  the current build references, and those binaries *are* current — so neither
  `--installed` nor `--maxsize` touched them, and the ceiling added for issue
  #1037 reported `applied 4096MB ceiling` over a 28GB tree. Measured on this
  repository: 28GB before, 672MB after.
