---
bump: minor
---

### Added
- A deterministic `text_formalization` module implementing Igor Martynov's *Formal protocol for translating texts into a knowledge base* "as is": the nine primitives (Concept, Entity, Predicate, Assertion, Procedure, Context, Temporal, Modal, Annotation) with their canonical JSON wire format, a structured Links-Notation view, a declarative conjunctive query over assertions, a curated «Сказка о рыбаке и рыбке» knowledge base exercising all nine primitives, and a constrained closed-class extractor that reproduces the article's worked example and never guesses off-template. Every primitive is additionally reduced to plain links/doublets, demonstrating that *everything is a link*. Surfaced through a `formal-ai formalize` subcommand (`tale` / `extract`) and a worked example, with a full case study, protocol→links mapping, and online research under `docs/case-studies/issue-468/` (issue #468).
