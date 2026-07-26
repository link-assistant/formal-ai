# Online research

Research performed 2026-07-25. Only primary GitHub documentation was used for
GitHub Actions semantics.

- [Workflow syntax: path filters](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore)
  states that `paths-ignore` skips a workflow only when all changed paths match,
  describes two-dot push versus three-dot PR comparisons, and warns about
  large-diff limits.
- The same [workflow syntax reference](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
  warns that a workflow skipped by path or branch filtering can leave its
  associated required check pending. This supports retaining a cheap detector
  job rather than suppressing the complete workflow.
- [Using jobs in a workflow](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-jobs#defining-prerequisite-jobs)
  explains that skipped dependencies propagate unless a job condition
  explicitly permits evaluation. This supports retaining `!cancelled()` for
  manual dispatch, where `detect-changes` is intentionally skipped.
- [Evaluate expressions](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#operators)
  defines `&&`, `||`, and grouping in job conditions. An unconditional true
  push term in the disjunction therefore makes later detector terms irrelevant.
- [Troubleshooting workflows](https://docs.github.com/en/actions/how-tos/troubleshoot-workflows#debugging-job-conditions)
  recommends inspecting expanded job-condition results when a job runs or
  skips unexpectedly. The preserved run metadata and logs provide the
  equivalent concrete evidence for this incident.

No third-party library is needed. The required operation is deterministic path
prefix classification over the file list already produced by Git, and keeping
it in the existing detector avoids another policy implementation.
