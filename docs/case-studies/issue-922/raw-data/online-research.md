# Online Research For Issue 922

Research was refreshed on 2026-08-13 from primary project and paper sources.

## DreamCoder: corpus-guided abstraction

[DreamCoder](https://arxiv.org/abs/2006.08381) learns a library of reusable
symbolic abstractions from programs that solve a task corpus. Its wake/sleep
architecture also uses a neural recognition model. Formal AI transfers the
symbolic corpus-compression idea only: recurring event-kind sequences become
parameterized algorithm candidates, and existing deterministic held-out tests
decide whether they generalize. The neural guidance component does not transfer
because this repository's reasoning core is explicitly non-neural.

## `stitch_core`: efficient abstraction discovery reference

The [`stitch_core` crate documentation](https://docs.rs/stitch_core/latest/stitch_core/)
describes a compression step that discovers abstractions and rewrites a corpus
with invented functions. It is the issue's external implementation reference.
The repository already has link-native subsequence compression in
`algorithm_discovery` and `sequences` with stable evidence identities and
held-out validation, so this slice reuses that implementation rather than
adding a second abstraction language or an unnecessary runtime dependency.

## Repository prior art

- [Issue #531 / PR #642](https://github.com/link-assistant/formal-ai/pull/642)
  introduced parameterized algorithm discovery, support traces, held-out
  validation, and inert learned records.
- [Issue #559 / PR #560](https://github.com/link-assistant/formal-ai/pull/560)
  made the executable method catalogue observable as link data.
- [Issue #656 / PR #690](https://github.com/link-assistant/formal-ai/pull/690)
  introduced trusted benchmark replay, append-only rejection evidence, explicit
  confirmation, and local review-branch materialization.
- [Issue #701 / PR #817](https://github.com/link-assistant/formal-ai/pull/817)
  demonstrated learned-seed adoption through the promotion boundary.
- [Issue #873 / PR #983](https://github.com/link-assistant/formal-ai/pull/983)
  applies the same immutable-candidate and stable-recovery principles to
  research learning.

The missing piece was therefore integration, not a replacement synthesis
engine: real recursive-core logs had to feed discovery, validated abstractions
had to feed promotion, and only the promoted result could feed the registry.
