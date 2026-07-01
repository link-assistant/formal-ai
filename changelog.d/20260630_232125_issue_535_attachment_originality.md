---
bump: minor
---

### Added
- Generalized the document-originality handler into a full verification class: authenticity, factual-accuracy, and veracity requests (not only plagiarism/uniqueness) now route to the same grounded workflow across English, Russian, Hindi, and Chinese.
- Weighed every extracted statement with relative-meta-logic (github.com/link-foundation/relative-meta-logic): statements start from an assumed-true prior, are raised by trusted original-first sources, lowered by contradicting originals, and unoriginal reposts are ignored — recorded deterministically in the append-only event log.
- Grounded each statement with a dedicated fact-check web-search query, mirrored byte-for-byte into the Web app worker so the browser matches the Rust engine.

### Fixed
- Routed multilingual text-attachment originality and plagiarism checks through a grounded attachment workflow instead of falling back to unknown.
- Included sampled text/plain attachment content in Web app solver context so browser uploads can be inspected by deterministic handlers.
- Folded Telegram document attachments into the shared attachment-context builder so forwarded files reach the same originality/verification handler.
