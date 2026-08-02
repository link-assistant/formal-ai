# Online research

Research was refreshed on 2026-07-16. Current product contracts come from
official documentation; the computational-model claim is grounded in research
literature rather than inferred from the implementation.

## Normal algorithms and computational universality

The University of St Andrews research repository preserves Eleftherios
Papathanassiou's 1979 PhD thesis, *On the equivalence of Markov Algorithms and
Turing machines and some consequent results*:
<https://research-repository.st-andrews.ac.uk/handle/10023/13736>.

The thesis gives constructive transformations in both directions: an arbitrary
Markov algorithm to an equivalent Turing machine, and an arbitrary Turing
machine to an equivalent Markov algorithm. That is the basis for describing the
ordered rewrite representation as computationally universal. It does not imply
that a resource-bounded execution accepts every computation; the production
step cap is a deliberate operational restriction.

Math-Net's primary-paper record for A. A. Markov's 1967 *Normal algorithms
connected with the computation of boolean functions* includes the English paper
and DOI: <https://www.mathnet.ru/eng/im2534>. It confirms normal algorithms as a
formal computation and complexity model, distinct from probabilistic Markov
chains.

## link-cli's substitution query language

link-cli (`clink`) is the priority dialect named in the issue-#715 review. Its
README is <https://github.com/link-foundation/link-cli>.

It states the model this work adopts:

> This tool provides all CRUD operations for links using single [substitution
> operation](https://en.wikipedia.org/wiki/Markov_algorithm) which is turing
> complete.

The hyperlink on "substitution operation" targets the Markov algorithm article —
the tool does not spell the words "Markov algorithm" in that sentence, it links
them. That link is the whole reason the query language and `src/normal_markov.rs`
can be the same object rather than two similar ones.

The README splits every operation into two sides:

> Each operations split into two parts:
>
> ```
> (matching pattern)
> (substitution pattern)
> ```

and derives CRUD from the shape of those sides, quoted verbatim:

| Shape                   | link-cli's own words                                            |
| ----------------------- | --------------------------------------------------------------- |
| `((1: 1 1)) ((1: 1 1))` | "this 'no change' can be used as read query"                    |
| `() ((1 1))`            | "Creation is just a replacement of nothing to something"        |
| `((1 1)) ()`            | "Deletion is just a replacement of something to nothing"        |
| `((1: 1 1)) ((1: 1 2))` | "the update is substitution itself, obviously"                  |

Those two middle rows are exactly the issue's requirement that "creation is
absence or empty or 0 length sequence substitution to non-empty sequence, and
deletion is reverse". The requirement is not an analogy to link-cli; it is
link-cli's documented definition.

The README also documents variables — "Where `$i` stands for variable named `i`,
that stands for `index`. `$s` is for `source` and `$t` is for `target`" — and
named references, `(child: father mother)`, persisted to a companion
`<database-name>.names.links` file.

### Both operand domains, one language

The dialect first shipped over text sequences only, on the reasoning that
link-cli's operands are links over an associative store while a code file is a
character sequence. Review feedback on #727 rejected that split — the ask was
link-cli's substitution patterns "for text sequences and links in general" — and
it was right to: the substitution model is the operand-independent part, so
supporting one domain and not the other is an incomplete generalization, not a
principled boundary.

`src/links_substitution_query/` is therefore a shared parser core plus one file
per operand domain. `parse_substitution_query` reads text; the operands are
quoted character sequences. `parse_link_substitution_query` reads links; the
operands are `(source target)` or `(index: source target)` doublets over
`link_store::DoubletLink`, whose `{index, from, to}` is already link-cli's exact
shape. `$i`/`$s`/`$t` carry over with them, binding across slots — a variable
used twice constrains the match rather than rebinding, so `($i: $s $s)` selects
exactly the links whose source and target agree. Both domains share the ordered,
restart-at-rule-zero, bounded control model, which is what carries the Turing
completeness argument across unchanged.

### The trace has to be readable, not just look readable

The mutation trace is published as the meta-language representation of the
request, so "it renders like Links Notation" is not the bar — a Links Notation
reader has to be able to read it back. It could not.

Links Notation escapes a quote by **doubling** it (`""`), and its encoder first
picks a delimiter the value does not already contain; `lino-objects-codec`'s
`escape_reference` carries `println!("hi");` as `'println!("hi");'`, with no
escape at all. Eight private `escape_lino_value` copies in `src/` had instead
converged on a C-style backslash escape, in four mutually incompatible
variants. A backslash leaves the quote visible to the reader, which ends the
string early — and once the string has ended early, a code fragment's `(` is no
longer inside a value but an unclosed group, so the document fails to parse
outright.

That is the difference between the two failure modes, and why this surfaced as
issue #715's title rather than as garbled output:

| value | backslash escape | codec's escape |
| --- | --- | --- |
| `Hello, world!` | survives | survives |
| `say "hi"` | **silently dropped** | survives |
| `println!("Hello, world!");` | **parse error** | survives |
| `fn main() {\n …\n}` | **parse error** | survives |
| `C:\path` | **mangled** | survives |
| `(("Hello")) ((terminal: "Goodbye"))` | **parse error** | survives |

`experiments/issue_715_lino_escaping_probe.rs` reproduces that table. Quoted
prose survived, which is why the defect stayed invisible: it needed a value
carrying both a quote and a paren — that is, ordinary code — to escalate from
silent loss to a hard error. Publishing the link-cli query in the trace made it
unconditional, because the query language always has both.

The fix is not a ninth escaping variant but the removal of one: `link_field`
delegates to the codec's own `escape_reference`, so the definition of the
notation and the encoder for it cannot drift apart again.
`tests/issue_715_notation.rs` holds the trace to that bar by parsing it with the
same codec the library encodes with.

### One document, two readers

The same divergence runs deeper than the trace, and measuring it changed the
fix. `SubstitutionRuleSet` and `AssociativePackage` both validate an incoming
document with the codec (`parse_indented`) and then read it with the
repository's own `src/seed/parser.rs`. So one artifact has two readers that
disagreed about escaping:

- `src/seed/parser.rs` — a C-style backslash dialect, which is what the
  hand-rolled escapers were written against.
- `links_notation::parse_lino` — the real grammar, which has **no** backslash
  escape and *doubles* a delimiter instead.

They agree whenever a value needs no escape, which is why this stayed invisible
for so long: it takes a value carrying a quote — that is, ordinary code — to
separate them. `experiments/issue_715_round_trip_dialects.rs` runs both dialects
past both readers, and the table has no winning column: the backslash escape is
rejected by the *grammar* on `println!("hi");`, and the codec's output is
rejected by the *seed reader* on a value carrying both quotes. No renderer
could satisfy both, so the reader had to move too.

It turned out the reader was simply wrong, not merely different. Links Notation
escapes a delimiter by doubling it, and `strip_comment` in that same file
**already** implemented the rule — only the value decoder never learned it. The
corpus is not single-dialect either: of 1434 `.lino` files under `data/`, five
already write the doubled form (`the subject''s name` in
`data/cache/wikidata/property/P138.lino`; also `dollar`, `dollars`, `painted`,
`does`) and three write the backslash form. The doubled files could not be read:
a value carrying the delimiter had no closing quote on its own line, failed to
decode, and fell back to raw text with the quotes still in it — live data
corruption, predating this issue.

That makes the reader fix additive rather than a migration. A single-quoted
value containing `''` had no valid meaning before, so giving it one cannot break
a valid document, and both corpus dialects are now read. This is why the
`43 files` figure recorded in an earlier draft of this document was wrong on
both counts — the number and the conclusion drawn from it.

With the reader speaking the notation, the writers could follow. Seven files had
each grown a private `escape_lino_value` — the same C-style escape, copied and
subtly varied (some escaped `\r`/`\t`, some did not; several slots were
interpolated with no escaping at all). They now share one encoder,
`links_format::format_lino_value`, which does not implement the rule so much as
borrow it: the codec's always-quoting encoder is private, so the helper formats
a one-field record with the public `format_indented_ordered` and takes the field
back off it. That costs an allocation per value and buys the property that
matters — it cannot drift from the notation, because it *is* the notation's
encoder. `tests/unit/issue_715_renderer_artifacts.rs` holds every renderer to
both readers at once; reverting the writers makes all four cases fail on
`println!("hi");`.

### The mirror drifts by the same mechanism

`tests/source/` is a hand-copied second library, wired as its own test binary,
whose purpose is to reach private functions (`docs/case-studies/issue-559`).
Nothing enforces that it matches `src/`, and it is only maintained where someone
remembers: measured file by file, **51 of 143** mirrored modules differ from
their source by more than two lines — `protocol.rs` by 907, `server.rs` by 517,
`anthropic.rs` by 485. Two modules that predate this branch, `method_registry`
and `cue_lexicon`, were never mirrored at all.

This is the same failure as the escapers, one level up: a rule with two copies
and no gate between them. The copies diverge silently, and the drift is only
discovered when something forces the comparison.

Ten mirrors are updated here — the ones this branch touches, including
`normal_markov`, whose divergence was this branch's own unmirrored commit.
`intent_formalization` is deliberately left at its inherited drift: syncing it
requires mirroring `method_registry` and `cue_lexicon`, which in turn need
`solver_dispatch` (173 lines behind), `solver_handler_how` (332 behind), and an
`include_str!` path that does not resolve from `tests/source/`. That cascade is
real work with no bearing on this issue, and the mirrored copy is dead code no
test reaches. It is recorded here rather than fixed quietly or left unsaid.

Two escapers were deliberately left alone, because they are not this rule:
`google_trends_catalog` collapses whitespace and is lossy by design, and
`memory` plus `links_substitution_query` are each a self-consistent
writer/reader pair over a different language (link-cli's *query* syntax, not
Links Notation).

### Deliberate divergences

Recorded rather than silently taken:

- **Terminal rules have no link-cli counterpart.** Normal algorithms distinguish
  terminating from continuing rules; link-cli has no such concept because it
  does not iterate to a fixed point. Rather than invent punctuation, the text
  dialect reuses link-cli's named-reference slot — the `child` in
  `(child: father mother)` — so a terminal rule is `(terminal: "text")`. The
  link dialect cannot reuse that slot, because there the pre-colon position is
  the link's index; link rules are therefore always non-terminal.
- **An unchanged store is not a state transition.** Over a set-valued store, a
  substitution that produces the link it matched is not selected. This is forced
  by link-cli's own documentation rather than chosen: it documents
  `(($i: $s $t)) (($i: $s $t))` as reading "all links without modification", which
  can only terminate and only mean that if the identity substitution is not a
  step. The same rule makes creation terminate, via link-cli's documented
  deduplication ("Identical sub-links are created once and reused") — otherwise
  `() ((1 1))` would append forever. Reads are consequently answered by matching
  (`LinkRewriteProgram::matched_links`), not by executing a rewrite.
- **Creation may not force an index.** link-cli's creation shorthand is
  `() ((1 1))`, whose two operands are the source and target; the index is the
  store's to assign. What `() ((5: 1 2))` should do when index 5 is already taken
  is undefined by the README, so it is rejected at parse time instead of guessed.
- **Over memory, the link dialect reads but does not write.** The dialect itself
  is complete — `LinkRewriteProgram::execute` creates, updates and deletes — but
  the surface that owns the links only exposes the read half. `formal-ai memory
  query` stores `MemoryEvent`s and derives the doublet view from them
  (`memory_events_to_link_records`), content-addressing each record id from the
  event's own canonical form; the projection is therefore one-way, and an edited
  link has no inverse back to an event. `LinkStore`'s only write is
  `append_memory_event` — an event, not a link. Accepting a link-level write
  would mean rewriting a projection nothing reads back, which reports success for
  a change the store never made, so the query is refused with a message naming
  the boundary. Giving the projection an inverse is a store change, not a query
  change, and is left as its own issue.
- **The two sides are not wrapped in an outer paren.** link-cli's README writes
  creation as `clink '() ((1 1))'` but its read-all example as
  `clink '((($i: $s $t)) (($i: $s $t)))'`, with an extra enclosing layer. The
  unwrapped `A B` form is the one this dialect took, for both domains; the
  wrapped variant is not accepted, since its intended meaning is not documented.

## LinksQL's confirmation of the model

LinksQL (<https://github.com/link-foundation/linksql>) is the second reference
named in the review. Its README states the computational claim directly rather
than by hyperlink:

> That single rule is Turing-complete (it is a Markov algorithm over an
> associative store), so it scales from one-line reads to complex multi-pattern
> rewrites without new syntax.

It frames the two sides as `(restriction) (substitution)` and derives the same
CRUD table, which independently corroborates link-cli's shape. Its "mixed" row —
"several substitutions in one statement" — is the multi-rule case this dialect
supports by pairing operands positionally.

One shape was **not** adopted: LinksQL admits a single-sided `(pattern)` as a
read. This dialect requires both sides, because a one-sided query is ambiguous
against an operand list and link-cli — the stated priority — always writes two
sides. A read is written the link-cli way, as an identity substitution.

## OpenCode tool and permission contract

OpenCode's official tool documentation is
<https://opencode.ai/docs/tools/>. It defines `read` as returning codebase file
contents, `write` as creating or overwriting files, and `bash` as running
commands in the project environment. It also documents that read, edit/write,
and command authority remain controlled by the client's permission settings.

This supports the implemented trust boundary: Formal AI plans capability-based
calls and consumes their results, while OpenCode owns filesystem and command
execution. The retained run also checks the locally installed CLI's actual
serialization, session continuation, read envelope, and tool names rather than
assuming the prose documentation is byte-for-byte protocol specification.

## OpenAI-compatible tool loop

OpenAI's official function-calling guide is
<https://developers.openai.com/api/docs/guides/function-calling>. It specifies a
multi-step loop: send available tools, receive a tool call, execute it in the
application, return the associated tool output, and continue until a final
response or further calls. The API reference additionally identifies
`tool_calls` as a finish reason and requires tool outputs to reference the call
they answer: <https://developers.openai.com/api/reference/resources/chat>.

Formal AI follows that contract without a CLI-specific filesystem side channel.
The same planner is exercised through Chat Completions, Responses, Anthropic
Messages, Gemini, the built-in Agent CLI, and OpenCode's OpenAI-compatible
provider.

## Related repository evidence

The raw snapshots for issues 680, 681, 712, 714, 715, and 716 are stored beside
this document. They establish the sequence from explicit file routing, through
agentic mode and typed execution recipes, to this issue's missing invariant: an
artifact established in an earlier turn must remain addressable, yet its current
contents must still be read from the client before mutation.
