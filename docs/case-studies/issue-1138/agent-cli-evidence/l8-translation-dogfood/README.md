# Formal AI translation-dogfood sessions (issue #1138)

External Agent CLI 0.26.0, local `formal-ai/formal-ai` model at
`formal-ai/0.351.0`, 2026-09-26. The server binary was built from the exact
PR #1139 merge content (commit `9c84a3af7`), so the projection behaviour
recorded here is the behaviour that merge landed. No hosted model participated.

## Sessions

Three authenticated sessions drove the local model through the Agent CLI's
native protocol (`http://127.0.0.1:<port>/v1`, private empty memory per run,
`FORMAL_AI_DREAMING=0`), all against plan 16's translation surface:

1. `ses_f222866f2ffeuDM0mB7i3bxYCT` — "translate this Rust function into
   TypeScript" with the code inline and unquoted. The planner misrouted: it
   read `current.is_empty` (a Rust method call from the prompt) as a file
   path, called `read`, and returned the file-not-found error as its final
   answer. Client and harness both exited zero; the misroute is the model's,
   not the transport's.
2. `ses_f2225a10fffexnf04Zq2Z0ELN3` — the same task phrased file-first
   ("read word_wrap.rs, write word_wrap.ts") with the write-capable tool
   surface advertised. The planner fell through (`task unrecognised`); no
   tool call was planned and no file was written.
3. `ses_f2221e2c7ffeOznESdzxXKZoAf` — the canonical phrasing the translation
   handler routes on: backtick-quoted code payload plus an explicit target
   language (`Translate \`...\` to TypeScript`). This routed correctly to
   `translate_rust_to_typescript`, and the chat program-translation path
   answered with a **translation gap**: the general chat path does not yet
   consult the plan-16 L8 grammar-projection rules. The full exchange is
   recorded verbatim in `translation-probe.md`.

## What the merged projection actually does

The positive and the refusal legs of the same binary, captured in
`translation-probe.md`:

- `formal-ai translate --from rust --to ts` on the plan-16 dialect probe
  renders typed TypeScript (`function dialect_probe(sealed: bool): u32 {`,
  exit 0) — the batch-4 type-position rules doing real work.
- The same leg on `rust/examples/basic_usage.rs` **refuses** with
  `use_declaration: the projection seed does not carry them, so nothing was
  written` — the splice-refusal surface PR #1139 landed, refusing loudly
  rather than emitting a lossy projection.

## Review

The honest reading of the three sessions: chat-phrased code translation is
not yet wired to the L8 projection — the intent routes, the answer is a gap.
The merged capability is reachable today through the translate CLI and the
driver's `translate` tool (source-root file level), which the probe leg
exercises end-to-end. Wiring the chat `translate_code` path to consult the
grammar-projection rules (and closing the read-tool misroute on unquoted
inline code) is open follow-up work for the issue #1138 corpus; nothing here
widens a gate or relaxes a check. Session transcripts and full logs remain
private at `/tmp/formal-ai-1138-evidence.twjioD`,
`/tmp/formal-ai-1138-evidence2.x86huk` and
`/tmp/formal-ai-1138-evidence3.DhxLxO` on the authoring machine.
