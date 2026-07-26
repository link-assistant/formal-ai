# Issue 874: documenting the associative technology stack

Issue [#874](https://github.com/link-assistant/formal-ai/issues/874) asked for a
separate Markdown document that links the associative component repositories
and briefly explains what each component is and how Formal AI uses it.

## Requirements translated into checks

The regression test in `tests/issue_874_docs.rs` turns that short request into
four executable requirements:

| Requirement | Evidence |
| --- | --- |
| R874-1: the guide is standalone and discoverable | `docs/associative-tech-stack.md` exists and `README.md` links it |
| R874-2: every direct associative dependency is grounded | all relevant `Cargo.toml` and `package.json` components have repository and usage text |
| R874-3: related repositories are not overstated | the guide labels direct, compatibility, in-repository, conceptual, and development-time boundaries |
| R874-4: the stack is explained end to end | the guide covers storage, notation, parsing, CST/AST, calculation, configuration, localization, substitution, and orchestration |

The test was committed before the guide. Its first run failed all four checks
because the standalone file and README link did not yet exist; the captured
output is [`agent-cli-evidence/red-test.log`](agent-cli-evidence/red-test.log).

## Research method

Repository membership was not treated as runtime integration. Each direct
component was checked against the current manifests and then traced into its
integration path:

- storage: `src/link_store.rs`;
- code and document structure: `src/coding/cst.rs` and
  `src/document_formats.rs`;
- calculation: `src/calculation.rs`;
- executable configuration: `src/main.rs`;
- browser localization: `src/web/i18n.js`; and
- in-repository query, substitution, and relative-logic modules for upstream
  projects that are architectural references rather than linked packages.

This produced the three-section status model used by the guide: direct runtime
components, architecture and protocol components, and development and
orchestration components. The model prevents catalog entries or related
repositories from being mistaken for compiled dependencies.

## Formal-AI-driven documentation leaf

The contribution workflow requires the real Agent CLI to drive Formal AI
through its public endpoint. The reproducible runner is
[`experiments/issue_874_agent_cli.sh`](../../../experiments/issue_874_agent_cli.sh).
It creates an isolated worktree, starts this repository's `formal-ai` binary,
points the published Agent CLI at that server, and verifies the requested file
byte-for-byte.

The first wording asked for both a heading and a paragraph. Formal AI
incorrectly classified it as an unrelated web-fetch task, and the request
failed on a 403. That failed attempt is retained in the top-level
`agent-cli-evidence` logs instead of being hidden.

The second request used the currently supported explicit file operation:

> Create file associative-stack-summary.md containing Formal AI stores
> inspectable knowledge as a links network with Links Notation as its portable
> text representation.

It succeeded in Agent session `ses_0600227bcffeTu1kE8xcHCnR63`. The resulting
[`agent-authored-summary.md`](agent-cli-evidence/explicit-containing/agent-authored-summary.md)
is included verbatim as the opening statement of the finished guide. The raw
Agent stream, stderr, Formal AI server trace, task, and worktree status are
retained beside it so the result can be audited rather than inferred.

To reproduce:

```bash
cargo build --bin formal-ai
experiments/issue_874_agent_cli.sh
cargo test --test issue_874_docs -- --nocapture
```

## Result

The standalone guide now gives readers both an end-to-end data flow and a
component-by-component map. Most importantly, it says where integration stops:
`doublets-web` is a browser compatibility target; `link-cli` and
`relative-meta-logic` have in-repository adaptations; `meta-theory` and
`transformer` are conceptual or comparison references; and Agent CLI and Hive
Mind are development-time tools, not shipped reasoning dependencies.
