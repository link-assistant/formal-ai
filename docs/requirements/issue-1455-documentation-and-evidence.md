## Issue #1455: Grounded integration documentation and affordable documentation CI

The 2026-10-29 instruction adds investigation of incomplete PR #1456. These
requirements supersede blanket main-build/release requirements for docs-only
pushes; mixed source changes and explicit manual production verification retain
their existing obligations.

- Document command-stream, web-capture and web-search as actual direct
  integrations. Cite manifests and executable consumers, distinguish native
  target/features from browser HTTP/package boundaries, and explain acquisition,
  ranking and command execution without claiming that search proves an answer.
- Keep optional ecosystem candidates separate from proven runtime dependencies.
  The Cargo/package manifests govern versions; do not maintain arbitrary counters.
- For automatic push and PR events, explicit documentation paths (`docs/**`,
  root Markdown, nested README/CHANGELOG files and changelog fragments) select
  lightweight documentation/workflow checks only. Unknown runtime Markdown and
  mixed code/docs changes retain normal CI. Docs-only changes must not trigger
  an automatic release, even when a changelog fragment is pending.
- Preserve all heavy jobs, manual dispatch controls and schedules. Maintain
  filters through an idempotent JavaScript transformation that preserves source
  comments and job bodies. Never represent a filtered job as an executed pass.
  Review required-check configuration before making a filtered workflow required;
  GitHub can leave such checks pending. Large diffs and unavailable GitHub diffs
  may bypass platform path filtering; do not promise savings in those cases.
- Reduce the public PR #1456 failure to a provenance-labelled fixture. Formal AI
  must collect explicit repository evidence before returning an unsupported
  work-item gap, formalize the observations and decompose the remaining work.
  Bound paths, observations and repeated actions; stop on failed, partial or
  policy-refused reads. Preserve JavaScript/native behavior parity in CI.
- Reading evidence alone does not deliver the original task. Record unbound
  requirements, implementation and independent verification as unfinished. The
  current bounded preflight is a capability improvement, not an autonomous
  documentation-authoring claim; general evidence-to-edit synthesis remains an
  explicit development gap.
