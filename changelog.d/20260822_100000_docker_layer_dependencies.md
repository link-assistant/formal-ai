---
bump: patch
---

### Fixed

- Build the Docker image's dependencies from the manifests alone, before the
  sources are copied. `COPY . .` preceded `cargo build`, so every file in the
  tree was part of the build layer's cache key and editing one `.rs` rebuilt all
  ~500 dependency crates — a 24-minute image build that gated the pipeline's
  finish by itself. Measured locally: the manifest-only layer builds in 1m48s,
  and the source layer then compiles `formal-ai` alone.
