# Issue 846: excluded CI paths bypassed on pushes

## Result

The release workflow now uses the change detector for both pull requests and
pushes. One path list in `scripts/detect-code-changes.rs` suppresses every
job-gating output for:

- `experiments/`
- `dev/log/`
- `docs/case-studies/`

Manual `workflow_dispatch` runs remain unconditional. Real source, manifest,
documentation, and workflow changes outside those paths retain their previous
flags. The unused `mjs-changed` output was removed.

## Reproduction and evidence

Commit `ff38e2ab221ef27df7ab4ecc779b9c7293cd7a11` added six files below
`experiments/`. GitHub Actions run
[30118611467](https://github.com/link-assistant/formal-ai/actions/runs/30118611467)
started at `2026-07-24T18:52:27Z`. Its detector correctly reported no code
change, but the direct-push alternatives made Secrets Scan, Lint, Test,
Coverage, both principal e2e jobs, Build Package, deployment, and deployed-page
e2e execute anyway. The complete 28,983-line log and run metadata are preserved
under `raw-data/`.

The minimal automated reproduction is
`tests/unit/ci-cd/issue_846.rs`. Before the implementation all three focused
tests failed:

1. six change-gated jobs contained an unconditional push alternative;
2. no common ignored-path policy governed typed outputs;
3. the ignored path/event matrix had no detector coverage.

After the implementation, the same focused suite passes. The detector's own
tests exercise `.rs`, `.md`, and `.mjs` examples under every ignored prefix for
both `push` and `pull_request`, plus a mixed shipping-change control.

## Timeline

| UTC | Event |
| --- | --- |
| 2026-07-24 18:52 | Excluded-only commit `ff38e2ab` triggers run 30118611467. |
| 2026-07-24 19:20 | Issue 846 records the measured mismatch and root-cause candidate. |
| 2026-07-24 19:27 | The unnecessary run completes after tests and deployment. |
| 2026-07-24 21:17 | Maintainer expands scope to four templates, full evidence, research, and upstream reports. |
| 2026-07-25 | PR 854 reproduces the defect, audits all CI files, fixes it, and reports all affected templates. |

## Root causes

1. **Event and change policy were conflated.** Every expensive job used
   `push || workflow_dispatch || detector-output`. Since a direct main-branch
   update is a push, detector outputs were irrelevant for the event where the
   exclusion mattered most.
2. **Classification used two input sets.** `any-code-changed` filtered
   excluded folders, but `rs-changed`, `toml-changed`, `docs-changed`, and
   `mjs-changed` inspected the raw file list. Any specialized flag could
   reactivate a job.
3. **The policy lacked an event/path matrix test.** Existing tests verified the
   Git comparison range, not which jobs execute after classification.
4. **The same inherited design exists in all four templates.** Their language
   differs, but each computes specialized flags from unfiltered files and uses
   unconditional push clauses in change-gated jobs.

## Why job-level gating was selected

`paths-ignore` would prevent the whole workflow from starting, but it would
duplicate the exclusion list and can leave required checks pending when GitHub
skips a workflow. Keeping `detect-changes` as the cheap first job provides one
reviewable policy and makes skipped jobs complete as skipped checks. It also
preserves explicit manual runs and release-only push logic.

## Template audit

The entire `.github/` and `scripts/` trees of each template were enumerated,
then every detection, path-filter, and push-condition match was inspected.
Revision-pinned file lists and matches are in `raw-data/templates/`.

| Template | Revision | Result | Upstream report |
| --- | --- | --- | --- |
| JavaScript | `b529903a75ee0dde8ce16f91e117ae7e2ee85356` | affected | [#113](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/113) |
| Rust | `24ec71a4710c57219883b85770c0a7a13abfda24` | affected | [#109](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/109) |
| Python | `0223db03391eaf4cdde426c08172283d11d30e48` | affected | [#40](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/40) |
| C# | `c6ea17b108f1f0add7a1df615c0192ce16c2e607` | affected | [#40](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/40) |

## Preserved data

`raw-data/` contains the issue, comments, PR state and all three PR feedback
feeds; original run metadata and logs; related merged-PR search results; all
four template revisions, CI file trees, relevant source matches, and filed
issue snapshots; and the online research note. These are captured records, not
synthesized logs.
