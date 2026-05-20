### Added

- Added chat commands to list and read behavior rules, list self facts, and teach dialog-local behavior overrides.
- Surfaced behavior rules as `When X then Y` (or `When X do Y`) statements grouped by topic in both the catalog listing and per-rule detail; the same grammar — and its Russian, Hindi, and Chinese translations — now records dialog-local overrides.

### Changed

- Expanded unknown-intent fallback text with self-contained Links Notation teaching guidance.
- Behavior-rule listing now groups entries by topic (Greetings, Farewells, Identity, Capabilities, Hello-world programs, Unknown fallback) and renders each row as a `When X then Y` statement; runtime rules appear in a dedicated `Dialog-local rules` section.
