# Issue 885: legal boundaries and context-aware documentation audit

Issue [#885](https://github.com/link-assistant/formal-ai/issues/885) combines a
legal/product question, source-selection research, a link-centric philosophy,
and a requirement that Formal AI fact-check relative statements in individual
and whole Markdown documents through the real Agent CLI.

The preserved decomposition is in [`requirements.md`](requirements.md), the
implementation sequence is in [`solution-plan.md`](solution-plan.md), the
accessible share metadata is in
[`raw-data/shared-conversation-metadata.md`](raw-data/shared-conversation-metadata.md),
and the source ledger is in
[`raw-data/online-research.md`](raw-data/online-research.md).

## Finding

The core/default Formal AI runtime is symbolic and does not require neural
inference. OpenAI-compatible endpoints are protocol compatibility, while an
external vendor model or Agent CLI is an explicit tool boundary. This does not
turn a product-positioning statement into legal permission: every hosted model,
output, dataset, and intended parameter update still needs its exact license and
contract review.

Likewise, a provider's output assignment can coexist with a restriction on
competing-model development. A contributor may dedicate only controlled rights
under the repository Unlicense; third-party rights and provider restrictions
remain. The legal guides and candidate matrices therefore fail closed and leave
the training registry empty.

## Audit root cause and repair

Previously the extractor audited one Markdown line at a time. Given “The engine
records sources. It checks them,” the second claim lost its meaning before
evidence weighting. The repair gives each statement:

- a document-local link from a reference surface to the closest preceding
  compatible grammatical subject;
- `resolved_text` for evidence matching and probability assessment;
- `references` that persist the antecedent statement id; and
- `contextual_posterior`, capped by the antecedent's probability.

References never cross document boundaries. Demonstratives used as determiners,
such as “This document,” are not rewritten. Ambiguous or unsupported cases stay
visible for human review rather than being guessed across files.

## Fact-checked philosophy

[`docs/philosophy.md`](../../philosophy.md) retains “AI = data + algorithm,”
“everything is a link,” linked transformation, and recursive decomposition as
design theses. It corrects the literal Markov claim: a complete normal Markov
algorithm can be computationally universal; one isolated substitution rule is
not. One substitution occurrence instead receives one canonical link identity
connected to its larger semantic network.

## Formal AI and Agent CLI evidence

The reviewed task has five smallest documentation leaves: architecture,
output rights, datasets, models, and audit dependency policy. Formal AI served
the real Agent CLI and authored the fifth byte-for-byte in session
`ses_042f7df82ffee5AmxMtLafPMw3`:

Repository fact checking must resolve local references before weighing evidence, cap dependent confidence by antecedent confidence, preserve the dependency as a link, and leave ambiguous claims for human review.

That text is the exact
[`agent-authored-audit-policy.md`](agent-cli-evidence/agent-authored-audit-policy.md)
artifact. The [`agent-cli-evidence`](agent-cli-evidence/) directory retains the
raw/normalized stream, server request trace, classified stderr, task, session
id, and resulting workspace status. Reproduce it with:

```bash
cargo build --bin formal-ai
experiments/issue_885_agent_cli.sh
```

No paired self-authorship trailer is attached to the manually authored code or
the other four documentation leaves.

The separate
[`statement-audit`](agent-cli-evidence/statement-audit/) evidence set records a
real Agent CLI session that invoked Formal AI's public command over two Markdown
documents. Its report preserves `resolved_text`, `contextual_posterior`, the
antecedent statement id, and the same-document resolution policy. Reproduce the
release-gated integration with:

```bash
cargo build --release --bin formal-ai
experiments/agent_cli_e2e/run_issue_885_statement_audit.sh
```

## Verification scope

The regression suite checks every R885 requirement and the composed solution.
Focused statement-audit tests cover local pronouns, exact resolved evidence,
dependent probabilities, file isolation, and demonstrative determiners. The
repository audit emits source locations, per-statement findings, dependencies,
and a whole-repository summary. The exact committed-tree run and its content
digest are retained in
[`repository-audit-summary.md`](repository-audit-summary.md) rather than
checking in the multi-million-line generated graph.

Those artifacts are an evidence graph, not an omniscient truth oracle. Current
legal terms and external facts require captured primary evidence and human
review; the audit can expose missing or conflicting evidence and relative
confidence, but it cannot create permission.
