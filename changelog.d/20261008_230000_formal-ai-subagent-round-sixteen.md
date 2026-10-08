### Added

- PR #1188 round 16:
  - **Requirement extraction (R1188-U20):** `js/agentic/crate/requirement_extraction.mjs` and its Rust twin list the requirements an issue states: units with a seeded obligation cue, units opening with a directive verb, task-list checkboxes, and items under a heading naming requirements, deduplicated by content-word overlap. The vocabulary (`data/seed/meanings-requirement-extraction.lino`) covers five languages. `scripts/measure-requirement-extraction.mjs` scores it on 138 cached issues against 1350 reviewed shard rows: recall 0.411, precision 0.406. The floors in `data/meta/text-capability-ratchet.lino` may only rise; gate `check_requirement_extraction`.
  - **Readable code (R1188-U23):** `scripts/check-readable-code.mjs` (gate `check_readable_code`) fails packed files and lines over 300 characters of code under `js/`, `ts/`, `rust/`, `scripts/` and `packages/`. Exemptions need a reason and may only shrink. `translate-es` now carries the JavaScript layout through the pivot, so every `ts/` twin keeps its source's lines, indentation and comments. Long JSX lines are wrapped. The meta-language projections stay exempt until link-foundation/meta-language#217.
  - **Response languages:** 1,272 ru/hi/zh/es response rows were added. The response-language debt falls from 382 to 0: the 16 machine-text templates (code, log lines) are marked `audience machine`, which the ledger generator and its Rust twin test exempt. `multilingual-responses-synthesis.lino` is split into web, procedural-how-to, installation and documents files.

### Changed

- The JS meaning lexicon reads its file list from `data/meta/seed-registry.lino`. The hand-kept list had missed `meanings-response-intents-handlers`.
- The web UI boundary ratchet counts non-whitespace characters, so wrapping a line no longer changes the measure.
- JS -> Rust translated items: 254 -> 263.
