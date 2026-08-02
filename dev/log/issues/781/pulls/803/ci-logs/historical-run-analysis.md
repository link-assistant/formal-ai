# Historical branch CI analysis

Run `29709092598` was created at `2026-07-20T00:10:19Z` for the bootstrap
commit `d78ef730f86a1c0a43b874c843958eed5f081281`. Its conclusion was `success`,
but it did not exercise the issue #781 implementation.

The complete 2,662-line log was downloaded to `run-29709092598.log` and read
in two bounded chunks (`1..1500` and `1501..2662`). The relevant terminal
lines are:

- line 2614: the pull-request merge comparison contained only `.gitkeep`;
- lines 2616-2620: every typed change flag was `false`;
- line 2621: `any-code-changed=false`;
- lines 2657-2661: those false values became the job outputs.

No `##[error]` marker occurs in the log. The green conclusion therefore means
that bootstrap change detection completed successfully; it is stale evidence,
not proof that the Rust changes, protocol adapters, native-client harness, or
release workflow pass. PR #803 requires a run created after the final pushed
SHA, and every non-passing job from that run must be downloaded and analyzed.
