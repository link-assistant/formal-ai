# Source representation and synchronization

The [2026-10-18 source-model doctrine](requirements/doctrine-readable-recursive-source-and-comment-meta-language-2026-10-18.md) governs coding and self-coding. The full recursive source model is a core capability, not a signature census or optional source-formatting utility.

The common meta language holds semantic concepts and their concrete language facets. A source document contains a recursively nested syntax tree whose leaves contain readable source text. Comments, spaces, indentation, delimiters and quote spelling belong to that tree. Parser-internal positions may assist construction, but persisted content must reconstruct itself without original source buffers, numeric node tables or stored offsets. A file's program contains its own actual numeric literals; this rule prohibits replacing content with numeric inventories.

Language-neutral concepts describe the program's declarations, references, types, effects and control flow. Language-specific syntax remains attached as a concrete facet. An unfamiliar construct remains represented exactly with an explicit semantic gap; preserving syntax alone does not prove that its meaning is understood or that it can be translated. A semantic edit must pass target binding, type, effect and operational checks before it can propagate.

## Comments are source and meaning

A comment retains its original readable text, delimiter, whitespace, source attachment and documentation role. Its semantic facet contains formal propositions, relationships, conditions, obligations, references and evidence. Formalizing a comment must expose each unresolved meaning rather than labeling the original prose as a completed semantic representation.

Restoring the original comment uses its concrete facet. Producing an English explanation uses its formal meaning and the naturalization rules. The target's comment-emission rule chooses JavaScript/TypeScript or Rust documentation syntax while preserving that meaning. Re-formalizing generated English must preserve negation, quantification, modality and referenced entities. Formatting equivalence alone cannot qualify this check. Code fragments in comments use nested source models, and links and examples retain their provenance.

## Rules and validation

Parsing, projection, editing, emission and synchronization rules are inspectable Links Notation. Rules name their source concepts, captures, preconditions, target language/context and failure conditions. Language rules run before repository-specific rules; missing rules and ambiguous ownership refuse explicitly. A renderer must never silently return source-language text as a successful translation to another language.

CI reloads persisted content without the source buffer, reconstructs the complete original source, and independently reparses it. It checks semantic projection separately. For edits, it verifies the expected preimage, rejects overlap/capture/ambiguity, emits every affected target, independently validates each target, and exercises the inverse. Held-out inputs cover Unicode, BOM, CRLF, final-newline changes, nested comments where supported, strings/templates/macros, typed declarations, effects and doc-comment naturalization. Rust builds and product execution run only in CI.

Source inventories and diagnostics are derived from content. Checked-in node histograms, line-number inventories and manually refreshed aggregate counts are retired as source-representation authority. Digests may authenticate content but cannot replace it. Existing census-derived planner lookups, collectors and tests must migrate to structural content and stable semantic references; until then they are legacy partial consumers.

The model must support meaningful edits to Formal AI itself and other code through the same operations. A retained supplied controller is execution evidence with zero autonomous-authorship credit. The long-term measure is independently useful coding work from compact task descriptions, with reproducible outcomes and trusted discovery of missing knowledge.

## The codebase is the main reasoning corpus

Formal AI formalizes its own complete code, documentation comments and requirements using the same general path offered for other codebases. Formal statements retain source/construct attachment, conditions, evidence and provenance. Relative meta logic evaluates dependencies and conditional probabilities; shared premises cannot be multiplied as independent evidence. Local code and documentation and trusted public internet knowledge contribute through this same evidence model. Representation fidelity, inferred confidence and observed useful behavior are reported separately.

Documentation and comments are naturalized in English, Russian, Hindi and Chinese. Formal AI selects wording jointly across these languages using measured meaning survival for every ordered round trip, both whole-document and part-by-part. A score preserves the original input terms and denominator, with missing knowledge explicit. It cannot improve by omitting difficult content, copying opaque prose or inserting padding.

Requirements are formalized as scoped obligations, including polarity, quantifiers and conditions. Actual implementation and behavior evidence determines delivered, contradicted, partial or unknown results. The latest authoritative instruction overrides an older genuine conflict for the same scope; prior compatible requirements remain. CI retains the conflicting statements, their evidence and revision ordering, then signals the authorized Formal AI repair path. Verified code may require a documentation change; code that violates current requirements must be repaired rather than described as correct. Unsupported proof obligations stay explicit.

Prefer maintained generators, declarative transformations and bulk automation for source and documentation changes. Ask actual Formal AI to perform tasks it can solve. When it cannot, retain the failed ask and record the missing general capability with its independent expected outcome; improve that capability instead of silently substituting a hand-authored per-task answer. Supplied controllers and recipes remain automation with zero autonomous-authorship credit.
