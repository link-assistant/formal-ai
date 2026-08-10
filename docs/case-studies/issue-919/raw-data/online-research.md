# Issue 919 online research

Research was performed on 2026-08-10 against primary specifications. It shaped
metadata and replay boundaries; no new runtime dependency was added.

## Provenance

[SLSA v1.1 terminology](https://slsa.dev/spec/v1.1/terminology) defines
provenance as attestation metadata describing how outputs were produced,
including the platform and external parameters. The researched-procedure
ledger consequently retains the query (external parameter), exact source
identity, executor, verification result, and reviewer instead of storing only
the learned operands.

## License declaration

The normative [SPDX 3.0.1 license-expression
grammar](https://spdx.github.io/spdx-spec/v3.0.1/annexes/spdx-license-expressions/)
defines simple identifiers, `LicenseRef` values, exceptions, and compound
expressions. The v1 source format requires `SPDX-License-Identifier`; `NONE`,
`NOASSERTION`, control characters, and syntax outside the bounded expression
alphabet are rejected. The field records the source's declaration for review;
it is not a legal conclusion by Formal AI.

## Cache replay

[RFC 9111](https://www.rfc-editor.org/rfc/rfc9111.html) defines an HTTP cache as
a local store of response messages and describes reuse of a stored response for
an equivalent request. Formal AI's existing source cache keys discovery/page
requests by URL and verifies stored body bytes against SHA-256. The learning
projection intentionally omits the transient cache-hit flag, so the live and
offline executions represent the same observation when query, ranking, URL,
fetch time, and bytes are identical.

## Repository research

- Issue #873 / PR #983 supplies the candidate, immutable-gate, recovery, and
  continuation reducer.
- Issue #896 / PR #912 supplies the published search/capture integration while
  retaining `CachedSourceClient` as the network authority boundary.
- PR #897 and issue #848 supply the review-gated bounded workspace procedure
  executor and content-addressed ledger precedent.
- Issue #916 / PR #966 completes E69's write-effect ladder, the explicit
  dependency named by issue #919.
- Issue #918 / PR #986 establishes the current minimal-core and metadata audit
  immediately before this implementation branch.
