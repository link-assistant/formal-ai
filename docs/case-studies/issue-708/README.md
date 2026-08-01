# Issue 708: bounded associative-memory programs and exact query languages

Issue: <https://github.com/link-assistant/formal-ai/issues/708>

Pull request: <https://github.com/link-assistant/formal-ai/pull/883>

Formal AI can now compile a reviewed natural-language memory request into a
language-independent Links Notation program, show that program before reporting
the result, and execute it against the same append-only event log used by the
CLI, server, and browser. The compiler is intentionally closed: a request that
does not match a complete seeded family returns `program_gap` instead of
inventing a step. The same memory now has exact SQL and GraphQL CRUD,
aggregation, and statistics surfaces plus a human-gated auto-learning bridge
from repeated natural-language examples. All three routes share one typed plan
and lower to the link-cli-compatible substitution algebra. See
[`query-languages.md`](query-languages.md) for the exact schema, safety model,
meta-language audit, and evidence.

The issue and its comments were read through the authenticated GitHub API. There
were no comments or image attachments, so no screenshot evidence was required.
This changes execution and trace semantics, not visual layout.

## Root cause

The earlier conversation-memory surface had two useful but insufficient paths:

- whole-memory natural-language writes could append or substitute one value;
- Links Notation exposed a deterministic, one-way projection for read queries.

Neither path represented a multi-step program. Selection, filtering, mapping,
effects, composition, permissions, and termination were implicit in handler
control flow. That made a request such as “rename only facts I contributed”
impossible to review as one artifact, and made recursive normalization unsafe
because it carried no explicit bound.

Implementation also exposed two subtler gaps. The first serializer printed
friendly `replace` and `when`/`do` blocks, but the parser initially trusted only
the lower-level steps, so editing the visible blocks did nothing. A test captured
that unchanged program id before the parser made the reviewable blocks
authoritative. Separately, `map_matches` was initially only traced; the copy and
aggregation behavior was inferred from later output kinds. A second red test
pinned map state as an actual input to the following `create` operation.
The repository-wide browser suite then found a routing collision: treating a
bare resource word such as `fact` as a program gap intercepted the established
fact-checking handler. Seeded multilingual set-scope cues now have to accompany
a resource cue before an unmatched request is classified as a program gap.

The follow-up review exposed a broader representation gap. meta-language could
validate SQL and GraphQL syntax, while Formal AI's event log and link projection
could execute substitutions, but there was no semantic object joining those two
layers. Agent-facing protocol requests also treated the provided memory only as
recall context, so a syntactically exact query fell through to an unrelated
answer. `MemoryQueryPlan` now provides one canonical operation/filter/
projection/aggregate model, and exact protocol reads execute against an
isolated snapshot while implicit writes fail closed.

## Compiler and interpreter

The production path is a bounded sequence:

```text
request
  -> normalize without discarding bound values
  -> match one complete seeded multilingual template
  -> bind placeholders into reviewed primitive steps
  -> canonicalize language-independent program + stable id
  -> serialize for review / optionally parse an edited form
  -> permission gate -> bounded execution -> append-only trace
```

The seed catalog owns both syntax and authority. Every primitive has one
permission class, and every family has ordered steps plus English, Russian,
Hindi, Chinese, and Spanish surfaces. Canonical identity includes the family,
bindings, bounds, primitives, permissions, and arguments but excludes source
language. Equivalent translations therefore compile to the same program id and
Links Notation.

| Primitive | Interpreter meaning | Permission |
| --- | --- | --- |
| `match` | Select active, non-retracted events by seeded fields. | read |
| `filter` | Narrow the current selection by role, kind, missing marker, duplicate content, or missing links. | read |
| `map_matches` | Carry a reviewed projection into the following effect, including grouping/counting and source-preserving collection copies. | read |
| `create` | Append a content-addressed derived event; repeated execution deduplicates it. | write |
| `update` | Apply bounded substitutions, tags, kind changes, or whitespace normalization to selected events. | write |
| `delete_with_retraction` | Append a retraction that names the source event and reason; never erase history. | destructive |
| `sequential_compose` | Preserve reviewed step order. | read |
| `bounded_iterate_to_fixpoint` | Repeat until an iteration changes nothing or the explicit cap is reached. | read |

`MemoryProgramLimits::from_decomposition_depth(d)` sets `max_iterations` to at
least one and `max_matches` to `32 × max(d, 1)`. Both values are serialized in
the program. An oversized selection halts before its following effect; a
non-converging program reports `iteration_limit` rather than claiming a
fixpoint. Destructive programs require `DestructiveConfirmed`, including a
reviewed program edited from `do create` to `do delete_with_retraction`.

## Operation census and related work

The census used `link-cli` commit
[`ab2ce8b`](https://github.com/link-foundation/link-cli/tree/ab2ce8be8e671c91e011d4f02eea10a19deea809).
Its documented algebra expresses CRUD as one restriction/substitution pair:
identical sides read, an empty restriction creates, an empty substitution
deletes, and differing sides update. Variables bind on the restriction side and
are reused by substitution; persistent transformations add `always`, `once`,
and `never` lifecycle modes. See the primary
[`link-cli` operation examples](https://github.com/link-foundation/link-cli/blob/ab2ce8be8e671c91e011d4f02eea10a19deea809/README.md#L91-L138)
and
[`link-cli` trigger documentation](https://github.com/link-foundation/link-cli/blob/ab2ce8be8e671c91e011d4f02eea10a19deea809/README.md#L352-L368).

Formal AI keeps the useful match/effect separation but adapts deletion to its
append-only memory contract. `sequential_compose` corresponds to explicit
operation ordering, while bounded fixpoint iteration is the safe analogue of a
persistent transformation.

The exact query layer applies the same census directly: `same -> same` is read,
empty restriction is create, differing restriction/substitution is update, and
empty substitution is delete. The typed plan and lowered CRUD effect are
revalidated immediately before execution. The complete mapping, exact SQL and
GraphQL subsets, all-field schema, statistics, and controlled learning process
are documented in [`query-languages.md`](query-languages.md).

SPARQL provides a useful triple-store comparison: the W3C update language
separates `INSERT`, `DELETE`, and ordered update requests, but natural language
still needs a semantic parser before those operations are available
([SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/)). Research systems
map natural language to SPARQL with learned models, including
[Modern Baselines for SPARQL Semantic Parsing](https://arxiv.org/abs/2204.12793)
and
[Neural Machine Translating from Natural Language to SPARQL](https://arxiv.org/abs/1906.09302).
This implementation chooses reviewed seed templates and all-or-nothing
compilation instead of claiming open-ended semantic parsing.

Boundedness is a substantive constraint, not a timeout label. Datalog research
shows that boundedness is decidable only for particular fragments and becomes
undecidable for broader variants; see Guessarian and Veloso-Peixoto’s
[boundedness analysis](https://academic.oup.com/logcom/article-pdf/4/4/375/6406554/4-4-375.pdf)
and the later
[decidability results for monadic programs](https://arxiv.org/abs/1406.7684).
The engineering inference here is deliberately conservative: do not attempt to
prove arbitrary synthesized recursion bounded; ship a finite reviewed algebra
and carry caller-derived caps in every executable program.

Within Formal AI, [PR 597](https://github.com/link-assistant/formal-ai/pull/597)
established whole-memory natural-language read/write,
[PR 779](https://github.com/link-assistant/formal-ai/pull/779) established the
shared event-log surface, and
[PR 815](https://github.com/link-assistant/formal-ai/pull/815) supplied the
all-or-nothing seeded procedure-compiler pattern reused here.

## Test-first reproductions

The durable regressions cover:

- one stable program for equivalent English, Russian, Hindi, Chinese, and
  Spanish requests;
- all 15 seeded families and every primitive;
- selective contributed-fact rename and termination at fixpoint;
- weekly topic and contributor aggregation, missing-label todos, and mapped
  collection copies;
- editable `replace` and `when`/`do` blocks that change execution and cannot
  escalate permissions silently;
- match and iteration limits, destructive refusal, append-only retraction, and
  honest `program_gap` output;
- the same compile, execute, persist, refuse, and gap behavior in Chromium over
  IndexedDB.
- exact SQL and GraphQL semantic identity for all CRUD shapes and seven
  aggregates, with all 15 memory fields available;
- common ANSI/PostgreSQL/MySQL/SQLite/SQL Server/BigQuery semantics, exact
  syntax and schema rejection, link-program drift refusal, and browser parity;
- automatic template inference that remains inert until a green held-out suite
  and explicit named human approval; and
- real Agent CLI execution of SQL and GraphQL through Formal AI's
  OpenAI-compatible server, including read-only refusal of implicit mutation.

The original red evidence is retained in the commit history: the first
compiler test preceded the public compiler, the visible-shape regression kept
the same id after editing `replace`, and the map regression could not find a
`collection_member`. Each is green in the final focused suite.

## Self-application

The task was decomposed into five review leaves: compiler invariant, seed
catalog, bounded execution acceptance suite, native implementation, and browser
integration. Formal AI through the real Agent CLI authored the first three
artifacts; the implementation and browser leaf were manually integrated and
are not claimed as tool-authored. This is three of five leaves (60%).

The first exact-file request exposed a general planner bug: an edit phrase
inside a literal payload could outrank the `with exactly this content` marker.
The marker now owns the payload, its unit regression precedes the fix, and
`experiments/agent_cli_e2e/run_issue_708.sh` replays that case through the real
server and Agent CLI in the required CI job.

- `ses_043a7e1faffe5cfXqj66PqBMja` authored the multilingual compiler
  regression under [`self-hosting-authorship/`](self-hosting-authorship/).
- `ses_0434f23f9ffeYTsz2Hr7nKduHX` authored the five-language primitive and
  15-family seed catalog under [`self-hosting-seed/`](self-hosting-seed/).
  The 15 routing-scope declarations added manually after the cross-feature
  regression are explicitly excluded from that byte-authorship claim.
- `ses_0434ed22bffekjueq9I2XscceW` authored the final acceptance suite under
  [`self-hosting-execution-tests/`](self-hosting-execution-tests/).
- `ses_041576852ffe163oFMQA9N8bIr` authored the red exact-language contract
  under [`self-hosting-query-languages/`](self-hosting-query-languages/). Two
  further Agent CLI sessions execute the implemented SQL and GraphQL surfaces;
  their raw streams and server trace are retained in that directory's
  `execution/` evidence.

`issue_708_self_hosting::captured_agent_artifacts_match_their_committed_leaves`
checks the generated bytes against the canonical leaves (allowing only the
final newline added to Rust source by `rustfmt`, and the disclosed manual
scope declarations in the seed catalog) and checks that every raw log used the
`formal-ai/formal-ai` model and contains a real session id. A later
self-referential attempt to put this evidence assertion inside the artifact it
verifies was not recognized by the current general file planner; it is not
counted as authorship. Moving the assertion to an independent test keeps the
claim reproducible without hiding that planner boundary.

## Verification

Focused exact-language Rust and browser-worker tests pass, including native
solver and Agent-protocol routing. Repository-wide Rust, documentation,
seed-closure, language-parity, file-size, hardcoded-language, worker-ratchet,
and Playwright gates are run again after merging current `main`; the PR records
their final status.

See [`requirements.md`](requirements.md) for requirement-to-test traceability.
