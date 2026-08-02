# Issue 739: production CI/CD audit

## Outcome

The failure was deterministic and reproducible. Pull-request CI validated Rust
compilation, Clippy, tests, and doctests, but not the public documentation build.
The release succeeded; only the later GitHub Pages job ran fail-closed `cargo
doc`, where two public-to-private intra-doc links failed. The fix moves the exact
documentation command into the pre-release lint gate, corrects both links, keeps
the deployment check as defense in depth, renames the production Pages workflow,
and makes advisory file-size annotations sensitive to net growth.

## Requirements

1. Reproduce and fix every diagnostic in the cited run.
2. Eliminate false-positive and false-negative CI signals without weakening hard
   checks.
3. Remove “demo” terminology from the production workflow.
4. Compare the complete repository/workflow trees with all four pipeline
   templates and report shared defects upstream.
5. Preserve raw evidence, research, timeline, tests, and verification in this
   case study.

## Timeline

| UTC | Event |
| --- | --- |
| 2026-07-15 00:54 | PR 727 opened. |
| 2026-07-16 19:49 | Commits `1424eb8d` and `127a30a7` added the two private intra-doc links. |
| 2026-07-17 02:56 | PR run 29542421202 passed; it ran doctests but no `cargo doc`. Six warning-band files were annotated. |
| 2026-07-17 06:05 | PR 727 merged as `5a6bd2d3`. |
| 2026-07-17 06:07 | Main run 29559067549 published successfully, then Pages failed in job 87822248958. |
| 2026-07-17 | Issue 739 investigation reproduced both rustdoc errors locally and added a failing policy regression before the fix. |

## Evidence and root causes

The cited Pages log reports `rustdoc::private_intra_doc_links` at
`src/intent_formalization.rs:76` and
`src/links_substitution_query/links.rs:249`. `RUSTDOCFLAGS=-D warnings` correctly
made both fatal. Turning off that lint or documenting private helpers would hide
broken public output, so the source prose now uses code formatting rather than
links.

The false negative was structural: `cargo test --doc` compiles examples, whereas
rustdoc-only link lints are evaluated while generating public documentation.
The old policy test inspected only the post-release Pages step, proving that a
failure would be detected too late. The new regression requires the first
fail-closed `cargo doc --no-deps --lib` to occur in `lint`, before `build` and all
publication jobs.

The six file-size warnings were also audited against PR 727's base
`31645a19`. Five files were already above 900 lines and did not grow (three
shrunk); only conversation memory grew, from 965 to 977. Changed-file filtering
therefore repeated baseline noise on incidental edits. Advisory annotations now
use `git diff --numstat` and appear only for net-growing warning-band files. The
repository-wide 1,000-line hard failure and all embedded-data checks still scan
the complete checkout.

## Four-template comparison

The exact shallow-clone heads and complete file trees are retained under
`raw-data/templates/`; the audit considered every listed file, then compared the
CI/CD-relevant implementations rather than copying language-specific mechanics.

| Template | Finding | Decision |
| --- | --- | --- |
| JavaScript | Comprehensive warning-policy tests; its unchanged-size warning defect is already tracked in issue 103. | Retain formal-ai's repository-wide hard gate and improve advisory relevance. |
| Rust | Documentation is built only in `deploy-docs`, without fail-closed rustdoc flags; it also has Cargo.lock, multi-OS, published-crate smoke, and resilient Buildx checks. | Reported the shared rustdoc gap as upstream issue 96. Existing issue 93 covers unchanged file warnings. The other release-hardening differences were analyzed in issue 736 and are unrelated to this Pages failure. |
| Python | Dedicated documentation workflow validates docs separately from publication. | Adopt the early validation principle, using Cargo's native command. |
| C# | Dedicated documentation workflow and workflow-policy coverage. | Adopt the policy-test principle, not .NET-specific tooling. |

Upstream reports:

- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/96
- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/93

## Primary research

- The [rustdoc lint reference](https://doc.rust-lang.org/beta/rustdoc/lints.html#private_intra_doc_links)
  explains that public links to private items render broken and are rustdoc-only.
- The [Cargo `doc` reference](https://doc.rust-lang.org/cargo/commands/cargo-doc.html)
  defines `--no-deps` and public/private documentation selection.
- [GitHub workflow triggering](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow)
  confirms that pushes made with `GITHUB_TOKEN` do not start another workflow;
  the release child commit therefore cannot supply the missing validation run.
- [GitHub status-check documentation](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/about-status-checks)
  establishes that pre-merge checks attach to commits, supporting validation
  before publication rather than relying on a later deployment check.

## Reproduction and verification

Before the fix:

```bash
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --lib
cargo test --test unit ci_cd::issue_739
```

Both commands failed for the expected reasons. After the fix they pass. The raw
directory contains the before/after command logs, full cited workflow log,
isolated Pages job log, preceding green PR log, GitHub API snapshots, and template
tree snapshots. Files over 1,500 lines are intentionally retained as raw evidence
and should be inspected in chunks or with targeted searches.

## Alternatives rejected

- Allowing `private_intra_doc_links`: produces broken public documentation.
- Running docs only after release: preserves the false-negative window.
- Removing file-size thresholds: discards a useful repository-wide safety gate.
- Annotating every touched warning-band file: recreates baseline noise and hides
  actual growth.
