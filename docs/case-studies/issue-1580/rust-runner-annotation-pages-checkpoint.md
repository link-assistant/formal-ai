# Rust template runner annotation pagination

The current pinned upstream helper paginates workflow jobs but reads only the
first check-run annotation page. The finite offline replay uses 101 annotations
and places the acquisition message at positions 0, 29, 30 and 100. The original
helper misses positions 30 and 100. A candidate uses `--paginate --slurp` and
flattens every annotation page; all six cases pass, including missing-message
and actual-code-failure exclusions. Disabling pagination fails five checks and
keeps the actual-code-failure exclusion passing. No request reruns a real job.

The fixture is copied byte-for-byte from upstream commit
`5f1e7a96acbf7a93b567195443cf6e5158fbbf1f`, with its MIT license and SHA256.
The candidate is confined to the replay and is not an upstream production fix.
Issue #226, merged recovery PR #227 and all three PR feedback endpoints were
read. An initial malformed CLI search failed; corrected paginated searches and
the failed request remain retained. No pagination issue was found before filing
[upstream issue #257](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/257).
Its published body was fetched again and compared with the exact draft. The
first final naming check rejected eleven abbreviated evidence filenames; full
English renames preserve all archived bytes and the failed check is retained.

All 151 maintained observation cases, the six focused cases and three affected
JavaScript gates pass. No Rust build or live runner-capacity claim is made. The
current downstream queue is not explained by this controlled reproduction.
Full template equivalence, adoption, native qualification of pending local
refactors and complete issue acceptance remain unfinished. Supplied scripts,
patches and conclusions receive zero independent authorship or reasoning credit.

Evidence: `experiments/issue-1580/evidence/checks/rust-runner-annotation-pages/`.
