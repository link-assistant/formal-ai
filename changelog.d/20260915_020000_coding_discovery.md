---
bump: minor
---

### Added
- Coding synthesis can now recognize HumanEval, MBPP, and multilingual conversational task shapes, discover licensed operations from Python documentation and Wikifunctions, compose verified Python programs, and remember provenance-bearing procedures without benchmark-specific bodies. Upstream benchmark runs opt into live discovery with `--online`, while offline replay remains the default.
- Coding synthesis now distinguishes callable functions from runnable programs, binds requested standard output through multilingual semantic slots, executes whole-program candidates in the bounded workspace, and carries every verified artifact through the shared agent write-tool contract.
- Abstract Wikifunctions recurrences are now discovered as typed expression trees, grounded through fetched operator descriptions, proven to descend toward a boundary, replayed against source testers, and rendered without embedding benchmark-specific function bodies. A rebuildable non-seed web cache carries the same formalization and multilingual Wikidata aliases to the browser worker.
- A generalized structural composer now covers arithmetic, collection, scan, ordering, pattern, predicate, symmetry, and weighted-grid families through source-grounded meanings and bounded example execution. The first 20 HumanEval tasks pass without a source cache, and the first 20 MBPP tasks pass when live discovery is enabled.
- Named integer sequences and source-defined recurrences can now be discovered through the official OEIS JSON API. The bounded catalog follows cross-references, accepts only a strict arithmetic or linear-recurrence grammar, derives index mappings from examples, preserves CC-BY-SA provenance, and replays through the shared content-addressed source cache.
- Rosetta Code example requests now return attributed GFDL examples, and explicit Rust execution requests run only in the bounded agent workspace.

### Fixed
- Structural coding meanings now use canonical nested surface facts and carry distinct official documentation groundings, preserving semantic-seed integrity without prompt-specific definitions.
- Conversational coding recognition now considers only the outer instruction after benchmark signatures and assertions are parsed, so programming words inside a fenced document cannot steal an unrelated document-conversion request.
- Pull requests that change the agentic routing subsystem now run the complete Agent, OpenCode, Claude, and Codex research replay before merge, closing the post-merge-only coverage gap from issue #1137 while unrelated branches retain the cheaper held-out gate.
- CLI repository roots are resolved to a stable absolute path before benchmark or summarization child workspaces run, so `--repository-root .` no longer turns a valid grader script into a duplicated nested path after `current_dir` changes.
- Python task recognition now selects the requested target definition after completed helpers, treats blank doctest output as `None`, and preserves periods inside quoted literals while splitting requirements.
- Coding-structure lookup now normalizes punctuation and tolerates one edit in long single-token concepts, so hyphenation and ordinary misspellings do not create artificial capability gaps.
- Agentic authoring now keeps read-and-author obligations together: it can derive caller-declared Links Notation fields from an inspected source record, write the result, and verify a decorated client read-back instead of ending after the input read.
- Agentic structured-document authoring now preserves repeated source-record cardinality, scopes values to each record, retains repeated exact fields, rejects ambiguous partial field-name matches, and accepts schema lists spanning semicolons.
- The general agentic planner now treats bare `with` as a literal file-content lead, allowing the real Formal AI Agent CLI self-authoring flow to preserve exact backticked multiline payloads.
- The self-authoring harness now waits explicitly for its local Formal AI server to bind instead of relying on platform-dependent curl retry behavior.
- The Rust lockfile now uses `rustls` 0.23.45, resolving RUSTSEC-2026-0285, and the dependency-audit proof parser uses extended `sed` expressions that work on both GNU/Linux and macOS.
- Live-link checking now excludes the byte-for-byte Python documentation captures used for coding-discovery replay, so expired links inside upstream fixtures do not fail repository documentation checks.
