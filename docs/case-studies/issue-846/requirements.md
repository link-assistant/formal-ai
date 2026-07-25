# Requirements inventory

## Original issue

- Excluded-only changes under `experiments/`, including Markdown and MJS, must
  not run tests, coverage, or e2e on a direct push.
- The same behavior must apply to `dev/log/**` and
  `docs/case-studies/**`.
- Exclusions must have one authoritative definition, or drift must be tested.
- Remove `mjs-changed` if no job consumes it.
- Preserve pull-request behavior except for correcting excluded-path leaks.

## Maintainer comment

- Read all issue and PR feedback and preserve issue-related data and logs.
- Compare the complete CI/workflow and script trees with the JavaScript, Rust,
  Python, and C# pipeline templates.
- Apply relevant template best practices across the repository.
- Report reproducible upstream issues wherever the defect also exists.
- Reconstruct the timeline, enumerate requirements and root causes, research
  alternatives/components, and propose a plan for every requirement.
- Add tracing only if evidence is insufficient.
- Complete all work in PR 854.

## Solver and contribution requirements

- Establish a minimal failing automated reproduction before the fix.
- Cover each requirement and the composed behavior with automated checks.
- Run focused and repository-wide quality checks.
- Preserve atomic history, update the draft PR metadata, merge current `main`,
  push only `issue-846-a407746d16fa`, mark PR ready, and verify fresh CI.

## Requirement-to-proof map

| Requirement | Implementation/proof |
| --- | --- |
| Ignored pushes skip expensive jobs | Push alternatives removed from six change-gated jobs; workflow policy test covers all six. |
| `.md`, `.mjs`, `.rs` cannot leak | `classify_changes` filters before every output; detector matrix test. |
| Three ignored roots | `CI_IGNORED_PATH_PREFIXES`; matrix test covers each root. |
| One authority | One constant is consumed before typed and aggregate classification; source-policy regression test. |
| Remove unused output | `mjs-changed` documentation and emission removed; test rejects its return. |
| PR behavior | Same classifier is event-independent; test runs the matrix for push and PR, while shipping controls remain true. |
| Manual behavior | Every affected condition retains `workflow_dispatch`; policy test asserts it. |
| Evidence/case study | Revision-pinned `raw-data/`, this analysis, requirements, plans, and research. |
| Template comparison/reporting | Complete tree captures and four linked upstream reports. |
