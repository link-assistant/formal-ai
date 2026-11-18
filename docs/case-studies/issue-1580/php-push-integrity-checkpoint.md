# PHP release publication results

This partial review supports [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The pinned PHP template is `3a034b27a56067971ca74dd4c74c7707056b215d`.
Its release CLI commits and tags locally, calls both Git push methods, then emits
`version_committed=true`. Those methods return unsuccessful process results that
the orchestration discards. Existing upstream tests intentionally use no remote
and accept the silent push failures.

Nine bounded controls execute the unmodified PHP entrypoint against disposable
local repositories and bare remotes. In both changeset and instant modes, an
update hook rejects a branch or tag. A rejected branch still allows a release tag
to reach the remote while its branch remains at the old commit. A rejected tag
leaves the branch updated but no remote release tag. Both original paths exit
successfully and emit the successful release outputs.

A disposable two-call candidate uses the existing throwing process helper for
both pushes. Both rejected operations become nonzero CLI outcomes without
successful release outputs; branch rejection stops the later tag push. A tag
failure still leaves the already accepted branch update visible. Four successful
controls retain exact version 1.2.4 in the remote tag before and after the candidate.
The candidate does not claim atomic publication or change the upstream repository.

The focused file passes nine tests; the maintained fast-PR entrypoint passes 52,
with zero skips/failures. Six affected JavaScript gates and whitespace pass. All
full logs were read and hash checked under
`experiments/issue-1580/evidence/checks/php-push-integrity/`. Eight additional source
fixtures are exact Git-blob/SHA-256-verified copies with the existing original
license. PHP runs with a 32 MiB limit; subprocess inputs and deadlines are finite.
No Composer installation, network publication, Rust build or PHP package build runs.

Formal AI's existing `scripts/version-and-commit.rs:920–989` already exits on
commit, exhausted branch-push and tag-push failures, and creates the tag after the
branch update. Those behaviors are retained. This new upstream report remains a
draft until immutable executable evidence is published and duplicates are checked.

The per-file release review records 17 completely read source/workflow/test files,
with explicit transfer decisions. The full 608-blob PHP semantic review, captured
CI tree review and broader issue criteria remain unfinished. Supplied commands,
candidate changes and conclusions receive zero independent Formal AI authorship credit.
