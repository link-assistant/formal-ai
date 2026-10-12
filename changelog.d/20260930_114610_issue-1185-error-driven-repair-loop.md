---
bump: patch
---

### Added

- The executor gains an error-driven repair loop for failed build and run
  steps (issue #1185): instead of stopping at the first failed step, the
  raw output is formalized into a structured diagnostic — file, line, error
  code, message — through one generic `{slot}` template matcher over the
  new `diagnostic_code_shapes` seed (14 languages, slots validated by
  shape, no per-language regexes in Rust), the diagnostic becomes a search
  query carrying the exact error code, each fetched source is read and a
  fragment retained only when it addresses that exact code or message, the
  retained fix is written as a Links Notation `repair_edit` record beside
  the artifact (`<stem>.repair.lino`, never a raw text patch), and the
  failed command is retried on a ladder bounded at three rungs whose limit
  is reported honestly when reached. Every attempt is reconstructable as an
  evidence chain (`repair_attempts` document) for the answer's derivation;
  when no fetched source matches, the loop reports the unresolved need by
  name and falls back to today's honest failure report rather than
  fabricating a fix. Delivered inert pending the main session's wiring:
  module registration, seed-registry entries, and the one-line consult in
  `command_reroute`'s failure branch (see
  `docs/requirements/issue-1185-error-driven-repair-loop.md`).
