# Readable source, formalized comments and a shared coding model

The owner clarified this direction on 2026-10-18 while delivering PR #1234:

- Use a full CST, preserving comments, whitespace and all source syntax.
- Store actual human-readable UTF-8 content recursively in Links Notation; avoid numeric dumps, flat token collections and count-only self-AST refreshes.
- Make the representation close to a language-neutral meta-language program, with explicit parse, emit, transformation and synchronization rules for JavaScript, TypeScript and Rust.
- Formalize comments as meanings and naturalize formal meanings into English documentation in each source language. Exact original text restoration and semantic naturalization are separate operations.
- Make these practices central to Formal AI changing itself and working with other code; fix contradictory vision, requirements, guidelines and implementation.

The current authority is [the standing doctrine](../requirements/doctrine-readable-recursive-source-and-comment-meta-language-2026-10-18.md). This note records instructions and research, not a claim that the current census meets them.

## Useful upstream mechanisms

[Meta-language PR #201](https://github.com/link-foundation/meta-language/pull/201), inspected at commit 49d790d56b402d5160bbf756238e28deccabc984, defines one common recursive concept space for grammars, complete CSTs, semantic annotations and transformation rules. Its [vision](https://github.com/link-foundation/meta-language/blob/49d790d56b402d5160bbf756238e28deccabc984/docs/vision.md) requires equivalence evidence that preserves recursive structure, binding, effects, ordering and operational context before concepts share identity. Source-free archives retain complete dependencies and ambiguity. Its [translation rules](https://github.com/link-foundation/meta-language/blob/49d790d56b402d5160bbf756238e28deccabc984/docs/translation-rules.md) supply recursive captures, rendering contexts, declared fallbacks and serialized rules. Reuse these mechanisms; do not confuse its legacy numeric-id/percent-encoded network serializer with the readable representation required here. The PR explicitly remains partial and does not establish full delivery for Formal AI.

[Router PR #761](https://github.com/link-assistant/router/pull/761), inspected at commit 1c34066c143b71bf46ae3d3a96aab97945bd8e05, contributes [translation rule phases](https://github.com/link-assistant/router/blob/1c34066c143b71bf46ae3d3a96aab97945bd8e05/docs/development/translation-rules.md) and [concrete-syntax edit discipline](https://github.com/link-assistant/router/blob/1c34066c143b71bf46ae3d3a96aab97945bd8e05/docs/development/concrete-syntax-bulk.md): reload serialized CST and checked semantics, independently reconstruct and reparse, apply exact nonoverlapping edits against expected preimages, preserve inverses, and compare typed JS/TS projections without losing comments. Its inspected flat token/span JSON and retained comment trivia do not implement this doctrine's recursive Links Notation or semantic comment naturalization. Adopt the invariants and improve the representation.

## Same-day continuation: code/doc facts and multilingual wording

The owner additionally requires the complete codebase and documentation comments to serve as the primary formalization/naturalization playground. Formal AI itself must translate documentation into English, Russian, Hindi and Chinese, choosing mutually close wording that maximizes round-trip meaning survival for complete documents and their parts. Code and documentation must stay formally synchronized and verifiable in CI. Relative meta logic must calculate statement probabilities with their actual dependencies and fact-check against both local code/docs and trusted public knowledge. Formalized requirements must be checked against implementation, documentation and other statements; latest authoritative conflicting requirements win within their scope. Contradictions are repair signals, and every code/doc conflict with current requirements must be fixable. The doctrine records these as R1188-CST13 through R1188-CST21.
