---
bump: patch
---

### Fixed
- The self-development status now runs on pull requests, not only on pushes to `main` and the daily schedule. Both `7f3d61fee` and `5b0973f65` merged green and left `main` red, because no pull request could see the check that would have caught it (issue #1113).
- Lowered the `non_kernel_rust_lines` ceiling from 112805 to its measured value 112713, the shrink since `v0.348.0` that the kernel ratchet requires of a release (issue #1085 D1.4).
