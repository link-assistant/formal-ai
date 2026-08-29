---
bump: minor
---

### Added

- Answer a question about a task's structure by thinking about the task. Nothing
  on the open web knows the caller's own work, so "Break the customer import
  rewrite into sub-tasks" no longer becomes a search for its own words: a new
  planner route puts the question to Formal AI's recursive decomposition and
  returns what it finds (#1066).
- Deliver an answer only the symbolic engine reaches. A request that asks for
  something to be found out *and* recorded at a named path used to deliver only
  what the agentic router produced, so every residual that needs no tool ended
  with nothing written (#1066).
- Read an English phrasal verb with its object in the middle. "Break the
  customer import rewrite into sub-tasks" and "break into sub-tasks" are the
  same verb, but only the contiguous form was in the lexicon — and it is the
  form a caller is least likely to write, because English puts a long object in
  the middle (#1066).
- Say why a decomposition produced nothing, in the caller's language. A task
  that is atomic, that states a single need, or that hit the depth bound now
  reports that reason instead of announcing a list and enumerating none (#1066).

### Fixed

- Never open for reading a file the same request asked to be written. The named
  path was the only file-shaped token in an evidence-record request, so it was
  read, the read failed, and the evidence file was never written (#1066).
- Never write the words that *name* a work product as its body. "Record the
  findings in `report.md`" states where the findings go, not that the file
  should contain the word "findings" (#1066).
- Never read a literal payload across a sentence boundary. A payload marker in
  one sentence and the file clause in another recovered everything in between,
  so a handover memo opened by instructing the reader to leave it somewhere
  (#1066).
- Never deliver a description of a pending web search as a finding, and never
  read an authoring sentence as a delivery destination — either one wrote prose
  about the wrong subject into the caller's file (#1066).
- Keep the text of a sub-task that ends in a question mark. The question-shape
  enforcement rewrote such a task to its bare marker, so a listed sub-task said
  nothing about what to do (#1066).
- Never throw the work away with the sentence that delivers it. "Break the
  customer import rewrite into sub-tasks and record what you work out in
  `import-split.md`" coordinates the work and its delivery into one sentence,
  and consuming the whole sentence as delivery left nothing to answer, so the
  request was answered in the transcript and the named file never appeared
  (#1066).
- Read a task from the colon that introduces it, not from the last colon in the
  prompt. "Break the warehouse restocking rewrite into sub-tasks. Deadline: the
  end of the quarter." made the deadline the task, and a deadline is an
  irreducible single need, so a rewrite that splits four ways was reported as
  unsplittable. The colon now counts only in the sentence that asks the
  question, which is the same sentence scoping already used to tell a command
  that is named from one that is ordered (#1066).
- Read a task from the block that asks it, not from the instructions addressed
  to the solver. A prompt that states its task, leaves a blank line, and then
  says how to work and where to leave evidence had that second block decomposed
  beside the task, so one listed sub-task was the framing sentences pasted
  together -- a numbered line a reader can do nothing with. The blocks that ask
  are the task, decided by the same recogniser that routed the prompt (#1066).
- Never let a calculation cue claim every word written after it. "Solve" is also
  ordinary English, and an embedded cue was read to the end of the prompt, so
  four unrelated sentences became one expression; they carried a digit and an
  `=`, so it looked evaluable, failed to evaluate, and answered anyway --
  displacing the decomposition the prompt actually asked for. A request is
  stated in a sentence, so its cue claims that sentence (#1066).
