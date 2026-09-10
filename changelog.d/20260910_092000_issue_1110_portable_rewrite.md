---
bump: patch
---

### Fixed
- Issue #1110: the rename command Formal AI emits runs on macOS. It was `sed -i 's/\bX\b/Y/g' -- FILE`, which is GNU-only twice over: BSD sed reads the script after `-i` as a backup suffix, and BSD sed has no `\b`. Every rename on a Mac therefore failed with `bad flag in substitute command`, left the file untouched, and was reported by Formal AI as its own verification failure — a confident failure report instead of a rename. The command is `perl -pi -e` now, which means the same thing on both. Verified by running ladder leaf 2.2.2.2.1 on macOS, where it now passes.
- The #1028 ladder's sparse checkout keeps `docs/` and one path under `dev/`: 31 `include_str!` sites compile files from `docs/case-studies/` and one from `dev/log/`, so excluding those trees made `cargo test` fail to compile inside a node — which the harness scored as the leaf's own failing tests rather than as its own breakage.
