# CI/CD template audit

The audit compared every file under `.github/` and `scripts/` with the current
JavaScript, Rust, Python, and C# AI-driven development pipeline templates. File
inventories, directory diff summaries, and the relevant full patches are stored
in `raw-data/`.

| Template | Fragment lifecycle | Same defect? | Result |
|---|---|---|---|
| [JavaScript](https://github.com/link-foundation/js-ai-driven-development-pipeline-template) | Changesets consumes pending changesets during versioning and stages the resulting tree. | No | No report required. |
| [Rust](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template) | Its automatic collector formerly had the identical omission. | Already fixed | [Issue 65](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/65) and [PR 66](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/pull/66) remove fragments after writing and stage `changelog.d` with `git add -A`. |
| [Python](https://github.com/link-foundation/python-ai-driven-development-pipeline-template) | Scriv collection deletes fragments by default and its add mode stages the changelog and deletions. | No | No report required. |
| [C#](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template) | The collector explicitly deletes consumed fragments and stages the changes. | No | No report required. |

## Relevant differences in Formal AI

Formal AI's manual Rust collector already matched the corrected template. The
automatic path had diverged and retained the old behavior. The fix follows the
latest Rust template semantics while adding local regression coverage and fatal
error handling for release-file staging.

The broader workflow differs substantially because Formal AI includes web,
desktop, Docker, dataset, self-coding, and deployment jobs not present in every
template. Those unrelated differences were reviewed but intentionally left
unchanged. The raw audit files make that scope decision reproducible.

