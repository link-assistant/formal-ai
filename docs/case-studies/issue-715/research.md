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

Two deliberate divergences in this dialect, both recorded rather than silently
taken:

- **Operands are text sequences, not doublets.** link-cli's operands are link
  indices over an associative store; a code file is a character sequence. The
  substitution model is identical, the operand domain is not. `$i`/`$s`/`$t`
  address a link's index/source/target, which a flat text sequence does not
  have, so they are not carried over here.
- **Terminal rules have no link-cli counterpart.** Normal algorithms distinguish
  terminating from continuing rules; link-cli has no such concept because it
  does not iterate to a fixed point. Rather than invent punctuation, this
  dialect reuses link-cli's named-reference slot — the `child` in
  `(child: father mother)` — so a terminal rule is `(terminal: "text")`.

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
