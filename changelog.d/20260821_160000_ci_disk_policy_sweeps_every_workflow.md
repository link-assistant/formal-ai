---
bump: patch
---

### Fixed

- Enforce the issue #534 disk policy across every workflow instead of three
  hand-listed files. `agentic-cli-matrix.yml` and `external-benchmarks.yml` were
  caching the multi-GiB `target/` tree — exactly what the policy forbids — because
  the guard never read them. Both stop, and the guard now sweeps
  `.github/workflows` so a new workflow cannot reintroduce it.
