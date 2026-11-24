# Git admission refuses invalid path encoding

The pull request admission classifier streamed NUL-delimited Git output through
StringDecoder. That API substitutes replacement characters for malformed UTF-8,
so an inventory could be accepted after silently changing its path names.

A real bare Git repository reproduces this with three different invalid byte
sequences in documentation-like paths. Before the repair, all three cases
returned success. The regression requires refusal before standard output or
GITHUB_OUTPUT receives any admission flags, for pull requests, dispatch and push.

The bounded inventory spool now uses a fatal streaming TextDecoder. Valid
literal BOM names, embedded newlines and Unicode characters crossing the
32-KiB read boundary remain byte-faithful. Existing verbose inventory counts
stay off unless FORMAL_AI_CI_VERBOSE is true, and temporary spools are removed
on failure.

Run `node --test --test-concurrency=1 experiments/issue-1580/large-change-admission.test.mjs`.
The existing maintained default-branch-build-policy entry imports these cases,
so the real Git regressions run in hosted CI. Child heaps are bounded at 128 MiB;
the largest existing inventory remains finite at 24,004 paths.

All six focused real-Git cases and all 184 maintained admission cases pass.
The complete JavaScript gate set has 89 passes, 54 explicit skips and no failures.
This source change requires fresh hosted qualification. It does not complete
the other template practices or the original issue.
