---
bump: patch
---

### Fixed

- List-files program answers now render language-aware sample output, so Python examples no longer show Rust fixture files like `Cargo.toml` or `main.rs` (issue #440). Browser responses also separate the "not run" status from the copy instruction and use a light code-block palette when the app is in light mode.
