# Issue 918 Online Research

## Semantic Metadata Shapes

- [FrameNet frame index](https://framenet.icsi.berkeley.edu/fndrupal/frameIndex)
  organizes situations as frames with participant roles (frame elements).
- [The Berkeley FrameNet Project](https://aclanthology.org/P98-1013/) describes
  the frame-semantic lexicon and its corpus-annotated examples. This supports
  making roles and concrete examples first-class metadata.
- [Wikidata data model](https://www.wikidata.org/wiki/Wikidata:Data_model/en)
  represents knowledge as statements whose properties have typed values and
  may carry qualifiers and references. This supports explicit typed metadata
  values and provenance rather than free-form side notes.

The issue's five fields are deliberately smaller than either external model.
They borrow useful shapes without importing a new runtime dependency or
claiming schema equivalence.

## Repository Prior Art

- Issue #699 and merged PR #877 established the earlier dispatch-method
  migration ledger and ratchet. The issue #918 source ledger retains it as
  history and adds recursive file coverage.
- `data/meta/recursive-core-recipe.lino` and the generic recipe interpreter are
  the existing data-plus-interpreter pattern used by the boundary.
- Issues #656 and #657 established gated promotion and the release
  self-hosting metric that this change preserves.
