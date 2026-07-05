---
bump: minor
---

### Added
- Issue #558 auto-learning (R558-08): `src/self_explanation.rs` answers "how does Formal AI work?" grounded in the system's *own* source, data, and tests rather than prose docs. Each topic cites real artifacts; every `CitationKind::Source` citation resolves its `content_id` from the owned manifest and *panics* if the path is not an owned source file, so a fabricated citation cannot be constructed. The rendered Links Notation is anchored to the whole-source manifest content id that the source-to-links round-trip proves lossless.
- Eighth agentic recipe (`src/agentic_coding/explain.rs`): the grounded self-explanation is reachable through the agentic interface, emitting `how-formal-ai-works.lino`. Like the source-graph recipe it commits no byte-pinned artifact because the citation ids track the whole source tree.
- `explain_formal_ai` example prints the canonical grounded explanation.
