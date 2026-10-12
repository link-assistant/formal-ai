# Conversation summary request ownership

The native function retains its concise source contract. The original fuller rationale is preserved verbatim below; moving four comment lines into this document keeps the existing boundary baseline without raising it. This is documentation relocation, not a claim of solver migration.

```rust
/// Recognise a request to summarize the running conversation by composing
/// meaning roles rather than matching raw per-language phrases (issue #386).
///
/// The universal algorithm is identical for every language: the prompt either
/// (a) carries a complete standalone conversation-summary phrasing, (b) carries
/// an objectless courtesy frame asking for a summary, (c) names a summary
/// directive *together with* a conversation reference, or (d) leads with a bare
/// summary directive (`summarize`, `резюме`, `总结`, …). The prompt is
/// re-normalised first so the boundary-aware matcher sees punctuation collapsed
/// to spaces. Mirror of `asksForConversationSummary` in the browser worker.
```

Declared source text is classified using the raw request head and payload framing. Conversation words inside supplied text cannot acquire ownership of actual dialog history. Existing bare/courtesy/conversation phrases and returning-user recap remain governed by loaded seed roles.
