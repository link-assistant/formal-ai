---
bump: minor
---

### Added

- Compose the document a request describes instead of transcribing its
  description. "Produce a final evidence note containing the selected tree
  level, node outcomes, test results, and session id" names the headings a
  finished note must have, not the bytes of a file; a new planner route reads
  the three seed-declared signals that say so (a composition verb, the noun for
  a document whose content is described rather than supplied, and a content lead
  followed by two or more enumerated parts) and returns the composed note. The
  note reports what was asked for and what the session actually observed, and
  says plainly when nothing backs a requested part (#1066).
- Answer a question about the repository by reading the repository. A request
  that says *inspect*, *examine*, *review* or *identify* without saying *search*
  now admits the workspace-search route, provided it names a code-shaped subject
  and no external source — so "check the current exchange rate" still reaches
  the open web (#1066).

### Fixed

- Never write a file that breaks a constraint the same request states about it.
  A literal write is only literal when it satisfies every stated constraint on
  the file it writes, so content recovered from prose is no longer written when
  the request also pins the file's opening line. All three literal-write routes
  share the guard, so the misroute cannot simply move (#1066).
- Stop reading a dotted run of digits as a file name. `1.1.1.1.1`, `2.7.19` and
  `192.168.0.14` all split on their last dot into a file-shaped stem and
  extension; a ladder node addressed by its path in the tree was opened as a
  file, failed with "File not found", and the run ended on a fabricated answer
  (#1066).
- Strip the sentence's full stop from a path token, so "Read the file
  `Cargo.toml`." is a read rather than a web search (#1066).
