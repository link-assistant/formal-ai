# Formal AI and language models

Reviewed on 2026-08-01. This document describes the repository at that
revision; it is not a claim about every future deployment or third-party
integration.

## The architectural boundary

Formal AI's core/default runtime is symbolic software. It does not contain or require neural-network inference, and this repository does not ship trained
neural weights. The response path uses inspectable Links Notation data,
deterministic handlers, substitution rules, the link store, and recorded
execution evidence. The machine-readable training registry remains empty.

The OpenAI-compatible, Anthropic-compatible, Gemini-compatible, and
Vertex-compatible HTTP shapes are protocol compatibility. They do not make the
runtime an implementation of those providers' model architectures. In the same
way, accepting an OpenAI-shaped request is no evidence that an OpenAI model
generated the response.

Formal AI can orchestrate an external Agent CLI, a vendor service, or a locally
installed neural model when an operator explicitly selects and authorizes that
tool. That external program or service is not a runtime dependency of the
symbolic engine. Its inputs, outputs, licenses, service terms, privacy boundary,
and provenance must remain separately identifiable. The integration must never
be described as Formal AI's own symbolic reasoning.

This is the practical distinction:

| Question | Core/default Formal AI | Optional external integration |
| --- | --- | --- |
| Where is learned behavior stored? | Human-readable rules, links, memory, and code | The external model may also use neural weights |
| Is neural inference required to answer? | No | Possibly, outside the core process |
| Is the OpenAI-shaped API architectural evidence? | No; it is a wire protocol | No; the selected route and model are the evidence |
| Can the external result update Formal AI automatically? | No | No; promotion remains test- and human-gated |
| Where are permissions recorded? | Repository rules and explicit capabilities | Those controls plus the model license and provider contract |

## Product-positioning rule

Formal AI is not developed as a large language model or as a thin product layer
whose essential answers come from a hidden LLM. Its differentiated architecture
is the inspectable associative network and deterministic transformation engine.
It may interoperate with or orchestrate language models as tools. A deployment
that silently replaces the symbolic answer path with a hosted model would cross
that boundary and must not retain the same architectural claim.

This rule is about accurate architecture and provenance, not a claim that
symbolic and neural systems can never expose similar user features. Comparable
features such as chat, code assistance, or an HTTP completion endpoint do not by
themselves establish the same implementation architecture.

## Verification map

- `src/solver.rs` implements the universal symbolic solver loop.
- `src/engine.rs` identifies the deterministic, no-neural-inference engine.
- `src/link_store.rs` provides the native associative store.
- `data/seed/` contains reviewable Links Notation knowledge and behavior.
- `data/training/source-registry.json` records that there are no approved
  parameter-updating sources.
- `docs/configuration/orchestration.md` documents explicit external-agent and
  vendor targets.
- `LEGAL-COMPLIANCE.md` controls source intake and external-model use.

The statement-audit command can check that these file references still exist,
but no repository tool can infer a provider's current legal terms from an old
document. Re-run the dated source review whenever a route, model, or purpose
changes.

## Non-negotiable controls

1. Identify the exact engine that produced each material output.
2. Keep external model selection explicit and opt-in.
3. Do not label vendor output as a symbolic Formal AI derivation.
4. Do not use external output for parameter-updating training without an
   approved source-registry entry.
5. Keep automatic learning as a proposal: tests and a human approve any durable
   rule or code change.
6. Follow the fail-closed process in
   [`LEGAL-COMPLIANCE.md`](../../LEGAL-COMPLIANCE.md) whenever rights or terms
   are uncertain.
