---
bump: patch
---

### Fixed

- Indexed the seed-network children read in `LinkStoreSource::from_store`, so
  booting the condition backend no longer scans every projected link per
  meaning child. A cold first completion on a 400-event memory store drops
  from ~17.7 s to under a second, and the projection-rebuild budget test
  passes again (issue #1138).
- Meaning interrogations ("what does X mean", "meaning of X") now surface the
  consulted-source record: the subject is read by the same seeded extractor
  the concept handler routes with, instead of the whole prompt becoming an
  unspecific focus that buries the record under the generic unknown guide.
- Restored the issue-report invitation for prompts the definition router
  claimed and still could not resolve ("explain X"), closing the issue 864
  regression. Raw unmatched prompts keep the plain teaching guide without the
  invitation, as the chat-surface specification pins it.
- HonestGap agent-mode requests no longer decline on the HTTP server surface,
  and dotted filenames (`alpha.txt`, `main.rs`) are recognized as request
  anchors, so the aider-style request reaches the planner again.
- Removed the document-generation action cues (给我/帮我) from the handler
  promotion table, restoring correct routing for Chinese sentences that
  mention rather than request those actions.

### Changed

- The ordered-list gate now honours an `inert` module list per declaration
  file (registry: `data/meta/merge-conflict-policy.lino`), so a plan drafted
  as tests before its leaf lands no longer blocks derived-artifact
  regeneration. The mirror stays exact for every compiled module, and a
  second unregistered module is still reported as drift.
