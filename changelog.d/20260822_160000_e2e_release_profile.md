---
bump: patch
---

### Changed

- Build the E2E harness binary without full LTO. `Build formal-ai (release)`
  took 536 seconds and compiled 510 crates from scratch, twice per pipeline, to
  produce a binary the agent-CLI harnesses only *run* — it ships nowhere, so the
  runtime speed LTO buys is measured by nothing in those jobs. Full LTO also
  defeats the compilation cache those jobs restore, since it defers optimisation
  into a single link-time unit that sccache has little to reuse. Measured
  locally on the same one-line source change: 198s with LTO against 42s without.
  The override goes through the environment rather than a named profile so the
  output stays at `target/release/formal-ai`, which seventy harness scripts
  hardcode. Everything that ships still builds with the unmodified `--release`
  profile, and a test pins that boundary.
