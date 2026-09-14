---
bump: minor
---

### Added
- Coding synthesis can now recognize HumanEval, MBPP, and multilingual conversational task shapes, discover licensed operations from Python documentation and Wikifunctions, compose verified Python programs, and remember provenance-bearing procedures without benchmark-specific bodies. Upstream benchmark runs opt into live discovery with `--online`, while offline replay remains the default.
- Rosetta Code example requests now return attributed GFDL examples, and explicit Rust execution requests run only in the bounded agent workspace.

### Fixed
- The general agentic planner now treats bare `with` as a literal file-content lead, allowing the real Formal AI Agent CLI self-authoring flow to preserve exact backticked multiline payloads.
- The self-authoring harness now waits explicitly for its local Formal AI server to bind instead of relying on platform-dependent curl retry behavior.
- The Rust lockfile now uses `rustls` 0.23.45, resolving RUSTSEC-2026-0285, and the dependency-audit proof parser uses extended `sed` expressions that work on both GNU/Linux and macOS.
- Live-link checking now excludes the byte-for-byte Python documentation captures used for coding-discovery replay, so expired links inside upstream fixtures do not fail repository documentation checks.
