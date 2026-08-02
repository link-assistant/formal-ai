# Exact and learned memory query languages

Issue #708 now has three user-facing routes into one memory-query model:

```text
reviewed natural-language template       exact SQL / exact GraphQL
                 \                           /
                  -> complete syntax validation
                  -> typed MemoryQueryPlan
                  -> canonical semantic identity
                  -> bounded link-cli substitution program
                  -> plan/program drift check
                  -> permission and match bounds
                  -> append-only MemoryEvent adapter + link trace
```

The route is intentionally split at the typed plan. Syntax acceptance is not
execution authority, and the SQL and GraphQL parsers do not each own a second
implementation of CRUD. Equivalent inputs produce the same canonical plan and
stable identity before lowering.

The browser does not carry a JavaScript copy of either grammar. The shipped
worker calls the same Rust tokenizer, SQL/GraphQL parsers, typed plan, and
semantic validator through WebAssembly, then a bounded Rust/WASM event adapter
applies the plan to browser memory. JavaScript only transports typed events and
persists an authorized result. Native builds additionally attach meta-language
CST and syntax-link evidence; the dependency is not currently `no_std`, so the
WASM parser evidence identifies the shared exact Rust parser instead.

## Schema and exact surfaces

Every field of the shared `MemoryEvent` record is addressable:

`id`, `kind`, `role`, `intent`, `tool`, `inputs`, `outputs`, `content`,
`sentAt`, `demoLabel`, `conversationId`, `conversationTitle`, `evidence`,
`accessCount`, and `writeCount`.

SQL accepts the `memory`, `memoryEvent`, or `memoryEvents` table names and the
following closed, exact subset:

- `SELECT`, `INSERT`, `UPDATE`, and `DELETE`, including `RETURNING`;
- `AND`, `OR`, `NOT`, parentheses, comparisons, `LIKE`, `CONTAINS`, and null
  predicates;
- projection, `GROUP BY`, `ORDER BY`, `LIMIT`, and `OFFSET`;
- `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, `VAR_POP`, and `STDDEV_POP`.

The common subset has the same semantics when callers select the ANSI,
PostgreSQL, MySQL, SQLite, SQL Server, or BigQuery profile. Those profiles
currently share meta-language's `sql-ansi` syntax grammar; vendor-only syntax
is rejected rather than guessed.

GraphQL accepts the read roots `memory` and `memoryAggregate` and the explicit
mutation roots `createMemory`, `updateMemory`, and `deleteMemory`. Its arguments
cover boolean `where` objects, projection, `orderBy`, `first`/`limit`,
`skip`/`offset`, `groupBy`, and the same seven aggregates. This is an exact
Formal AI memory schema, not a promise to interpret arbitrary application
schemas. Variables must be bound before execution so the executable text and
the reviewed text cannot diverge.

Examples:

```sql
SELECT kind,
       COUNT(*) AS count,
       SUM(accessCount) AS accesses,
       VAR_POP(accessCount) AS accessVariance
FROM memory
WHERE conversationId = 'demo'
GROUP BY kind
ORDER BY kind ASC;
```

```graphql
query {
  memoryAggregate(
    where: { conversationId: { eq: "demo" } }
    groupBy: [kind]
    orderBy: { kind: ASC }
  ) {
    count
    accesses: sum(field: accessCount)
    accessVariance: variance(field: accessCount)
  }
}
```

The two examples have identical executable semantics. Non-count statistics are
limited to the numeric usage counters, counter mutations require non-negative
integers, and non-finite numbers fail validation.

## Substitution semantics and bounds

Every compiled plan carries `max_matches` and `max_iterations`. Lowering emits
the existing `LinkRewriteProgram` representation used by the link-cli-compatible
algebra:

| Memory operation | Restriction | Substitution | Link effect |
| --- | --- | --- | --- |
| read | a matched link | the same link | `same -> same` / read |
| create | empty | a field/value link | create |
| update | an old field/value link | its new field/value link | update |
| delete | a matched link | empty | delete |

The compiler renders that program as Links Notation and immediately parses it
again in regression tests. Before execution, it independently verifies that
every emitted rule has the CRUD effect required by the typed plan and that its
step cap is exactly `max_matches × max_iterations`. A tampered plan/program pair
halts with `program_gap` without changing memory.

The link program also executes directly over the doublet projection of real
memory events in the acceptance suite. The typed event adapter remains the
authoritative persistence boundary because the current projection is one-way:
updates append or update events through `MemoryStore`, and deletes append a
`retraction` event rather than erasing history. The compiled substitution and
its observed effect are retained in the trace.

This design does not call an individual substitution, this bounded query
subset, ANSI SQL, or GraphQL “Turing complete.” Computational universality
belongs to the general ordered Markov/link-substitution system when unrestricted
composition and iteration are available. User memory queries deliberately carry
finite bounds and a closed schema, so arbitrary nontermination is not smuggled
into a chat request. The exact languages expose all current memory CRUD,
aggregation, and statistical operations while lowering into that universal
algebra.

## Authorization and surface parity

Reads require `ReadOnly`; inserts and updates require `Write`; deletes require
`DestructiveConfirmed`. An authorization failure is an auditable
`permission_denied` outcome, not a partial write. The native CLI/server solver
and browser worker both route exact inputs before seeded natural-language
programs. Browser mutations use the same permission result before IndexedDB is
persisted. Exact inserts and updates are explicit write requests; exact browser
deletes fail closed because the current worker message has no destructive
confirmation capability. Native callers can supply `DestructiveConfirmed`.

OpenAI-shaped request handlers receive protocol memory as an immutable snapshot.
They therefore execute exact reads against an isolated copy and refuse implicit
writes. Explicit native memory surfaces can supply write or destructive
authority, while the browser persists authorized non-destructive effects. This
preserves one query language without making an ordinary model request an
implicit database administration endpoint.

Parser, validator, lowering, and learning failures use stable diagnostic codes
such as `unknown_memory_field` and `permission_denied`. The CLI's generic
unrecognized-query sentence is selected from the multilingual seed rather than
embedded in Rust. This keeps exact-language diagnostics auditable without
adding new hardcoded natural-language debt.

## Controlled auto-learning

Learning generalizes repeated successful natural-language/exact-query pairs
without silently changing production behavior:

1. At least two successful examples using one exact dialect are syntax-checked
   and lowered.
2. The learner infers a one-slot natural-language template and a corresponding
   exact-query template.
3. Re-instantiating every observation must preserve the same placeholder binding
   and canonical semantics.
4. The candidate remains inert until a named held-out regression suite has at
   least one pass and zero failures.
5. A named human must explicitly approve promotion.
6. Future captures are restricted to safe scalar/identifier fragments, then the
   instantiated exact query passes the full parser and semantic validator again.

The promotion policy, candidate id, gate suite, and reviewer are serialized as
Links Notation. Seeded natural-language memory programs remain available for the
15 reviewed multilingual families; auto-learning adds reviewed exact-plan reuse
rather than an open-ended production semantic guesser.

## meta-language audit and upstream work

Native builds first ask meta-language for a full-match CST, reconstructed source
identity, and syntax-link evidence (`sql-ansi` or `GraphQL`). Formal AI then uses
its local typed semantic adapters because the audited meta-language v0.56.0
(`17fc9591dd48c1f5240d87baa1b535ada122b8fe`) exposes syntax grammars but no
shared executable query IR. The public registry is sufficient for this local
layer, so the PR is not blocked on a dependency change.

The generalized gaps are recorded upstream:

- [meta-language #187: shared executable SQL semantic IR and vendor normalization](https://github.com/link-foundation/meta-language/issues/187)
- [meta-language #188: GraphQL operations lowered into the shared executable query IR](https://github.com/link-foundation/meta-language/issues/188)

The exact submitted proposal text is retained beside this document in
[`upstream-sql-semantic-query-ir.md`](upstream-sql-semantic-query-ir.md) and
[`upstream-graphql-semantic-query-ir.md`](upstream-graphql-semantic-query-ir.md).

## Reproducible evidence

The red contract was authored by Formal AI through the real Agent CLI and is
retained with raw client/server logs in
[`self-hosting-query-languages/`](self-hosting-query-languages/). The canonical
test allows only `rustfmt` formatting differences from the captured artifact.

`experiments/issue_708_agent_cli/run_exact_query_execution.sh` starts a real
Formal AI server over a fixture memory file and asks two independent Agent CLI
sessions to execute SQL and GraphQL tasks through the OpenAI-compatible API. It
asserts the exact result data, server POSTs, and session ids and preserves the
streams under `self-hosting-query-languages/execution/`.

The durable regression matrix covers:

- SQL/GraphQL canonical parity for CRUD and every aggregate;
- all 15 memory fields, scalar types, boolean filters, sorting, and pagination;
- common-SQL profile identity;
- syntax/schema rejection and numeric validation;
- read/write/destructive authorization and append-only retraction;
- Links Notation render/parse round trips, doublet execution, bounds, and drift
  refusal;
- inert learning candidates, held-out and human promotion gates, and injection
  refusal;
- native solver and Agent-facing protocol routing; and
- browser parser, executor, effect, and result parity through the shipped
  Rust/WASM artifact, instantiated by a Node-driven contract test.
