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
