---
bump: minor
---

### Fixed

- Compose a document that a request specifies, instead of writing the
  specification into the file. "Produce a final evidence note containing the
  selected tree level, node outcomes, test results, and session id." names a
  document and its parts; "containing" is also the marker that introduces a
  literal payload, so the words after it were taken for the bytes and the
  request's own wording was written back as the answer. The sentence around the
  marker decides which reading applies, and the recogniser that decides is the
  one the composing route already uses (#1066).
- Answer the work a request states, not a label it carries. A request that
  reads "Atomic task 9: Assemble an intake summary containing the applicant
  name, the referral source, and the interview date." was answered "yes, that is
  atomic" -- true, and a reply to the heading rather than to the sentence after
  the colon, because the heading alone carries the atomicity predicate and the
  task noun. Naming something to produce states work to do, so the
  task-structure route stands aside for it (#1066).
- Stop a calculation cue at the end of its sentence whether a blank line follows
  or not. The bound was whichever the search found first, and it looked for the
  blank line first, so a cue in a paragraph with another paragraph after it
  still claimed every sentence up to the break (#1066).
