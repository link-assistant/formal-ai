---
bump: patch
---

### Security
- Update `rustix` from 1.1.5 to 1.1.6 in `rust/Cargo.lock` for RUSTSEC-2026-0486 (unsound xattr functions with a zero-length buffer), which made the denied-warning `cargo audit` fail on `main` and on every pull request. Only the version and checksum change; every other locked dependency stays as it was.
