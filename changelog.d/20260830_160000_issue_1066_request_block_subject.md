---
bump: patch
---

### Fixed

- Read a request's subject from the block that states the work rather than from
  the note that places the worker. A prompt handed to a worker often ends with a
  paragraph saying where they are and how to report; scored as part of the
  request, the longest code-shaped word in that paragraph won, so twenty of the
  twenty-nine searches the #1066 ladder planned looked for `binary_tree` -- a
  word out of the framing -- instead of the data model, atomicity check or
  execution adapter the node was asked about.
- Stop reading a permission to use the web as a statement that the answer is on
  it. "Use web research when it materially improves factual accuracy" sits in
  that same paragraph, and matching it across the whole prompt disqualified
  every ladder node from looking at the repository it had just been handed: six
  proof files recorded an open-web query assembled out of the framing, ending in
  "the tool returned no content", as their evidence. The workspace admission and
  the external-source veto that guards it are now both read at the scope of one
  block (#1066).
- Judge a proof whose body reports that a tool returned no content as hollow.
  An empty tool result is a step that did not happen and proves as much as an
  empty file, so `experiments/issue_1066_ladder_offline/judge-proof.py` now
  refuses it. A search that ran and matched nothing is an observation about the
  workspace and still passes (#1066).
