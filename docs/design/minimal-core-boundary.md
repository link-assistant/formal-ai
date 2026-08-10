# Minimal compiled-core boundary

Status: accepted for issue [#918](https://github.com/link-assistant/formal-ai/issues/918).

Formal AI keeps compiled code only for capabilities that cannot be expressed as
knowledge links without first having an executor for those links. The boundary
has four parts:

1. **Meta algorithm.** The bounded search, scoring, proof, learning, and
   self-improvement loop that selects and executes data-defined methods. Its
   implementations include `meta_core`, `meta_reasoning`, `meta_construction`,
   and `meta_self_improvement`.
2. **Link store.** Durable append, query, projection, and synchronization of the
   associative memory. `link_store`, `memory`, and `memory_sync` provide this
   substrate; facts and policies stored in it are data, not core code.
3. **Generic interpreters.** Parsers and bounded executors for seed rules,
   memory programs, substitution queries, recipes, and proof programs. An
   interpreter may implement syntax, limits, effects, and authorization, but it
   may not embed a domain vocabulary, a canned answer, or a domain routing rule.
4. **Host surfaces.** Thin CLI, server, WASM, browser, filesystem, process, and
   network adapters. A surface translates host I/O and enforces host safety; it
   does not own user-facing language or problem-domain policy.

Everything else is seed or learned data. In particular, intent phrases,
domain-specific recognition, handler precedence, response prose, worked
examples, units, preconditions, effects, and problem-solving policies belong in
`data/seed/` or memory. Generated views and caches may compile that data for
delivery, but they do not become an independent source of truth.

## Promotion test

A compiled component is inside the boundary only when all of these are true:

- it implements one of the four core parts above;
- its behavior is domain-independent after seed and memory inputs are removed;
- moving it to links would require recreating the same interpreter or host
  primitive in links; and
- the handler ledger names the core component and records a reviewable reason.

Mixed files fail the promotion test. They remain migration debt until the
generic primitive is split from domain recognition, policy, and rendering.
Deleting a handler is preferred when no behavior depends on it.

## Handler burn-down ratchet

[`data/meta/core-boundary-ledger.lino`](../../data/meta/core-boundary-ledger.lino)
is the complete recursive census of `src/solver_handlers/**/*.rs`. Each row has
one audited disposition:

- `migrate`: compiled domain knowledge must move to seed rules; the current file
  and aggregate outside-core line counts are ceilings;
- `promote`: the file passes the promotion test and carries a core component and
  reason; or
- `delete`: the retired path must be absent and remains recorded as an audit
  decision.

[`scripts/check-minimal-core-boundary.rs`](../../scripts/check-minimal-core-boundary.rs)
rejects unledgered nested files, missing live files, resurrected deleted files,
line growth, stale line baselines, and incomplete decisions. A reduction must
lower the reviewed baselines in the same change, turning every reduction into
the next ceiling. The older issue #699 method ledger remains useful for dispatch
migration history, while this source ledger closes its non-recursive census gap.
