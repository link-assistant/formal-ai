---
bump: patch
---

File-size checks now fail when maintained text cannot be read or the scan cannot
complete. JavaScript and Rust preserve the same errors and existing exclusions,
and evaluate exclusions relative to the checkout so a parent named `target`
cannot hide oversized source files.
