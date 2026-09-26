# Translation probe record — formal-ai/0.351.0, 2026-09-26

Verbatim record of the translation surface of the formal-ai binary built
from the PR #1139 merge content (`9c84a3af7`), as driven in sessions
`ses_f222866f2ffeuDM0mB7i3bxYCT`, `ses_f2225a10fffexnf04Zq2Z0ELN3` and
`ses_f2221e2c7ffeOznESdzxXKZoAf`.

## Session 3 final answer (chat, canonical phrasing)

Prompt: `Translate \`<word_wrap rust function>\` to TypeScript` — routed to
`translate_rust_to_typescript`; the model's final message:

```text
Translated `<fn word_wrap(text: &str, width: usize) -> Vec<String> { ... }>`
from rust to typescript:

```typescript
// translation gap for `<the rust source>` from rust to typescript
```
```

The chat path answered honestly with a translation gap rather than a lossy
projection; the plan-16 L8 rules are not yet consulted by
`translate_program`.

## CLI leg — render (exit 0)

Command: `formal-ai translate --from rust --to ts --input <probe.rs>`

Input:

```rust
fn dialect_probe(sealed: bool) -> u32 {
    if sealed { 7 } else { 41 + 1 }
}
```

Output (verbatim, byte-for-byte as printed by formal-ai/0.351.0):

```typescript
function dialect_probe(sealed: bool): u32 {
if (sealed) {
7
} else {
41 + 1
};
}
```

## CLI leg — refusal (exit 1)

Command: `formal-ai translate --from rust --to ts --input
rust/examples/basic_usage.rs`

```text
Error: "translate rust → ts refused use_declaration: the projection seed
does not carry them, so nothing was written"
```

The splice-refusal surface PR #1139 landed: an unmapped construct refuses
loudly and writes nothing, rather than emitting a silently lossy projection.
