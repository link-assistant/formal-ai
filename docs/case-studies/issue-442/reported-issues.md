# Upstream issues reported (R3)

Issue #442 asks that, where the same CI defect exists in the shared pipeline
templates, it be reported upstream with a reproducible example, a workaround, and
a fix suggestion. Three of the four templates share the bug; the **python**
template is the correct reference and needs no report.

| Template | Affected? | Reported issue |
|---|---|---|
| [rust-ai-driven-development-pipeline-template](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template) | Yes — direct source of formal-ai's pipeline (`needs.changelog.result == 'skipped'`) | <https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/71> |
| [csharp-ai-driven-development-pipeline-template](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template) | Yes (`needs.changeset-check.result == 'skipped'`; also missing `!cancelled()`) | <https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/25> |
| [js-ai-driven-development-pipeline-template](https://github.com/link-foundation/js-ai-driven-development-pipeline-template) | Yes (all fast-checks skip together on non-code changes → `test` still runs; no positive change gate) | <https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/79> |
| [python-ai-driven-development-pipeline-template](https://github.com/link-foundation/python-ai-driven-development-pipeline-template) | **No — correct reference pattern** (gates `test` on `py-changed / tests-changed / package-changed / workflow-changed`) | — |

Each report contains the root-cause chain, a reproducible example (a one-line
`.gitkeep`/`*.md` change that triggers the full suite), a workaround, and a
concrete `if:` fix that gates the `test` job on the change-detector outputs.
