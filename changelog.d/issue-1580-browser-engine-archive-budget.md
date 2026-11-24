---
bump: patch
---

Check the packed browser engine's existing 128 MiB archive budget before
artifact upload or npm publication, and report the largest files when the
archive exceeds the limit.
