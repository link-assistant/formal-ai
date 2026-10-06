# Architecture

This document describes the evolving architecture of `formal-ai`. Where
`VISION.md` captures *why* the project exists and `GOALS.md` captures *what*
counts as success, this document captures *how* the runtime is structured and
how each piece talks to the others. It is the canonical reference for new
contributors who want to understand the full pipeline without having to
triangulate between five other files.

Issue [#103](https://github.com/link-assistant/formal-ai/issues/103) names
this document as the single source of truth for the design — the structure and
wiring recorded here; status lives in `ROADMAP.md`, per-requirement status in
`REQUIREMENTS.md`, and numbers in the ledgers — and asks for the
following ideas to be explicit:

- last input + previous messages + memory + user data form the system context;
- the input is translated to Links Notation and recorded in memory before the
  formalization step;
- each verb phrase is formalized as a Wikidata P-ID and each noun phrase as a
  Wikidata Q-ID (with `wikipedia` / `wiktionary` URLs as fallbacks);
- multiple candidate interpretations are scored and selected with a
  neural-network-style **temperature** knob;
- close-probability candidates either ask the user a clarifying question or
  guess, depending on configuration;
- reasoning steps can nest, so tool-generated reasoning (e.g.
  `link-assistant/calculator`) is recorded as a sub-trace of the parent
  reasoning step;
- everything is appended to a growable memory backed by the `link-cli` library
  (file-mapped `doublets-rs`) / `doublets-web`, with regular backups to browser
  storage and disk in `.lino` files;
- the local memory is treated as a cache of the public-knowledge database
  (Wikipedia, Wikidata, Wiktionary);
- the associative store supports stored transformation/substitution rules
  expressed as data, Rust/JS handlers, dynamically compiled Rust/JS, or
  natural-language skills convertible on demand;
- once an expression is formalized, the same engine translates between
  natural languages and between natural and programming languages.

The rest of this document is a table of contents: each bullet point is
expanded in a topic file under `docs/architecture/`, which also links each
idea to the source modules that implement (or will implement) it.

---

## Contents

Each topic file keeps the original section numbers, so references such as
"§16" or "ARCHITECTURE.md §18" still resolve through this table.

- [System Context, Pipeline, and Links Notation Input](docs/architecture/system-and-pipeline.md)
  - [1. System Context](docs/architecture/system-and-pipeline.md#1-system-context)
  - [2. Pipeline Overview](docs/architecture/system-and-pipeline.md#2-pipeline-overview)
  - [3. Translating Input to Links Notation](docs/architecture/system-and-pipeline.md#3-translating-input-to-links-notation)
- [Memory, Public-Knowledge Cache, and Fact Queries](docs/architecture/memory.md)
  - [4. Memory: Doublet Links, .lino Backups, and the Public-Knowledge Cache](docs/architecture/memory.md#4-memory-doublet-links-lino-backups-and-the-public-knowledge-cache)
    - [4.1 Local in-process event log](docs/architecture/memory.md#41-local-in-process-event-log)
    - [4.2 Dreaming maintenance planner](docs/architecture/memory.md#42-dreaming-maintenance-planner)
    - [4.3 Research, learning, and stable recovery](docs/architecture/memory.md#43-research-learning-and-stable-recovery)
    - [4.4 Default native link-cli / doublets-web store](docs/architecture/memory.md#44-default-native-link-cli--doublets-web-store)
    - [4.4 Public-knowledge cache](docs/architecture/memory.md#44-public-knowledge-cache)
    - [4.5 Fact-query reasoning pipeline (Issue #127)](docs/architecture/memory.md#45-fact-query-reasoning-pipeline-issue-127)
- [Formalization, Temperature, and Symbolic Probability](docs/architecture/formalization-and-selection.md)
  - [5. Formalization](docs/architecture/formalization-and-selection.md#5-formalization)
  - [6. Temperature-Based Interpretation Selection](docs/architecture/formalization-and-selection.md#6-temperature-based-interpretation-selection)
  - [6.1 Symbolic Probability Evidence](docs/architecture/formalization-and-selection.md#61-symbolic-probability-evidence)
    - [Evidence count and counted-utility ranking (issue #449)](docs/architecture/formalization-and-selection.md#evidence-count-and-counted-utility-ranking-issue-449)
- [Universal Problem Solver, Nested Steps, and Rules](docs/architecture/problem-solver.md)
  - [7. Universal Problem Solver](docs/architecture/problem-solver.md#7-universal-problem-solver)
    - [Minimal Compiled Core (Issue #918)](docs/architecture/problem-solver.md#minimal-compiled-core-issue-918)
    - [7.1 Project lookups and summarization](docs/architecture/problem-solver.md#71-project-lookups-and-summarization)
  - [8. Nested Reasoning Steps](docs/architecture/problem-solver.md#8-nested-reasoning-steps)
  - [9. Transformation and Substitution Rules](docs/architecture/problem-solver.md#9-transformation-and-substitution-rules)
- [Translation Between Languages](docs/architecture/translation.md)
  - [10. Translation Between Languages](docs/architecture/translation.md#10-translation-between-languages)
    - [10.1 Formalize → Meaning → Deformalize Pipeline](docs/architecture/translation.md#101-formalize--meaning--deformalize-pipeline)
    - [10.2 Resolution Order and Browser Fallback](docs/architecture/translation.md#102-resolution-order-and-browser-fallback)
- [Configuration, Event Log, Surfaces, Evidence, and Testing](docs/architecture/runtime-and-surfaces.md)
  - [11. Configuration](docs/architecture/runtime-and-surfaces.md#11-configuration)
  - [12. Append-Only Event Log](docs/architecture/runtime-and-surfaces.md#12-append-only-event-log)
  - [13. Surfaces](docs/architecture/runtime-and-surfaces.md#13-surfaces)
  - [14. GitHub Evidence Collection](docs/architecture/runtime-and-surfaces.md#14-github-evidence-collection)
  - [15. Testing Architecture](docs/architecture/runtime-and-surfaces.md#15-testing-architecture)
- [Audit History and Current Gaps](docs/architecture/audit-history.md)
  - [16. Audit History And Current Gaps](docs/architecture/audit-history.md#16-audit-history-and-current-gaps)
- [References](docs/architecture/references.md)
  - [17. References](docs/architecture/references.md#17-references)
    - [Domain background (symbolic AI)](docs/architecture/references.md#domain-background-symbolic-ai)
    - [Symbolic world models and contexts (issue #649)](docs/architecture/references.md#symbolic-world-models-and-contexts-issue-649)
    - [Usage-weighted associative persistence (issue #686)](docs/architecture/references.md#usage-weighted-associative-persistence-issue-686)
- [Module Map](docs/architecture/module-map.md)
  - [18. Module Map](docs/architecture/module-map.md#18-module-map)
- [Self-Development Release Loop](docs/architecture/self-development-release-loop.md)
  - [Formal AI self-development release loop](docs/architecture/self-development-release-loop.md#formal-ai-self-development-release-loop)
