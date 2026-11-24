### Fixed

- Reuse one resolved version snapshot when a recipe attaches and validates its generated CI workflow. A live-to-cache transition within the same planner call no longer disables the owned compiler-failure CI fallback. Keep original provenance and exact source ownership checks; cross-request attempt retention remains unfinished.
