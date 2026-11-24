# PHP input and process ownership checkpoint

The pinned template at 3a034b27a56067971ca74dd4c74c7707056b215d and fresh source
b61e334703d93a15e08bf1a278e0df0a9d6e98a8 have identical version-guard,
release-note and budget-wrapper blobs. The maintained fifteen-case input test
executes actual PHP without installing packages or making remote Git requests.
Seven bounded process-group cases separately compare the pinned wrapper with
the current Formal AI counterpart.

- Version comparison converts malformed, missing and wrong-type evidence into
  successful unchanged/bootstrap decisions. Checked fetch/commit/path controls
  in the disposable candidate reject unknown evidence while retaining genuine
  first-file, unchanged-version and ordinary changed-version behavior.
- Release-note truncation removes UTF-8 continuation bytes but leaves the leading
  byte, including when the original cut ends at a complete character. The actual
  default 120000-byte cap reproduces invalid UTF-8. Caps smaller than the notice
  also exceed their requested limit. The candidate preserves tested boundaries;
  arbitrary Unicode notices and URLs remain outside that qualification.
- All seven original process controls leave a worker alive at wrapper exit.
  Expired parent-success/failure cases return 0/7 rather than 124. The existing
  template test kills the worker itself after expecting success. Formal AI's
  wrapper passes the same seven unchanged controls. No privileged process is
  launched; Windows and EPERM behavior remain unqualified.

Upstream reports, with immutable reproduction links:

- [Version guard](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/31)
- [UTF-8 release notes](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/32)
- [Additional evidence on existing process ownership issue](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/15#issuecomment-7054888090)

The two reports and the comment establish reporting, not upstream repair. The
complete PHP tree comparison and runtime/practice acceptance remain unfinished.
All supplied commands, candidates and reviews retain zero independent authorship
or reasoning credit. PR #1581 remains draft and unmerged.

Run the maintained reproducer from the Formal AI repository:

```sh
node --test --test-concurrency=1 experiments/issue-1580/template-php-input-integrity.test.mjs
```

Original/candidate logs, bounded input definitions and source hashes are retained
under `experiments/issue-1580/evidence/checks/php-input-integrity/`. Actual posted
bodies, URLs, paginated deduplication observations and command records are under
`experiments/issue-1580/evidence/checks/php-input-integrity-upstream-publication/`.
