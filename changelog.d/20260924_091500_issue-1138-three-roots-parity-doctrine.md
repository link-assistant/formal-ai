---
bump: patch
---

### Added
- The three-roots doctrine is now standing requirements: full parity between
  the Rust, JavaScript and TypeScript implementations of the client and the
  entire backend server and all other logic, each translatable into the others
  through the meta language (REQUIREMENTS.md assembled from the new
  docs/requirements shard, the architect note
  docs/architect-notes/2026-09-24-three-roots-full-parity-via-the-meta-language.md,
  docs/meta-algorithm.md, docs/source-roots.md, VISION.md and the
  requirements-traceability index). Rationale recorded with it: JavaScript
  executes faster than Rust compiles, so js/ts can carry the iteration cycle
  once parity holds, and `formal-ai translate` is the vehicle for moving code
  in any direction without hand porting.
