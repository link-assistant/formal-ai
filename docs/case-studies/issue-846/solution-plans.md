# Considered solutions and execution plan

## Alternatives

### Trigger-level `paths-ignore`

This is concise and avoids even the detector job. It was rejected because the
same path policy would live in YAML and Rust, mixed changes still require
classification, and GitHub documents that path-skipped workflows can leave
required checks pending.

### Remove only the unconditional push clauses

This fixes the measured push bypass but leaves `experiments/README.md`,
`experiments/repro.rs`, and similar files able to set specialized outputs. It
does not satisfy the Markdown/MJS or single-authority requirements.

### Filter each output independently

This fixes both symptoms but repeats filtering at every flag and invites the
same drift when a new output is added.

### Filter once, then classify (selected)

Create one ignored-prefix constant, derive one relevant-file view, and calculate
every job-gating flag from it. Keep the broader documentation/changelog/example
rules only for `any-code-changed`. Remove push alternatives from change-gated
jobs, retain manual dispatch, and leave genuine release/deployment push
conditions untouched.

## Executed plan

1. Capture issue, PR, comments, reviews, the original run metadata, and its full
   log.
2. Enumerate this repository's complete CI file tree and inspect every detector
   output and push condition.
3. Clone all four templates at recorded revisions, enumerate their complete CI
   trees, inspect matching logic, search for existing reports, and file four
   reproducible issues.
4. Add failing workflow-policy, detector-policy, and whole-matrix regression
   tests.
5. Centralize ignored paths, filter all outputs, remove the unused output, and
   remove only the six inappropriate push alternatives.
6. Run focused tests, detector tests, formatting, lint/static policy, and the
   full relevant suite.
7. Review the complete diff, synchronize `main`, update PR 854, push, mark
   ready, and inspect fresh CI timestamps, SHAs, and any non-passing logs.

No additional debug output was added: the original job expression, detector
output, commit diff, and complete run log make both root causes deterministic.
