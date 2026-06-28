# Case study - Issue #497: GitHub repository traffic visibility

- **Issue:** [#497](https://github.com/link-assistant/formal-ai/issues/497)
- **Reported version:** 0.205.0 (WASM worker)
- **Reported prompt:** `можно ли узнать заходил ли кто либо в твое репо на github?`
- **Reported result:** `intent: unknown`
- **Pull request:** [#586](https://github.com/link-assistant/formal-ai/pull/586)
- **External references:** [`raw-data/online-research.md`](./raw-data/online-research.md)

## Requirement

The solver should recognize repository-traffic visibility questions by meaning,
not by the exact Russian sentence. The class combines four dimensions:

- GitHub as the repository platform.
- A repository noun or abbreviation.
- A traffic, visitor, view, clone, or referrer signal.
- A question frame asking whether the information can be known or seen.

The answer must be source-backed and multilingual. It should distinguish
aggregate GitHub traffic data from the identity of an individual visitor.

## Root Cause

The reported prompt named GitHub and a repository, but no existing route
classified GitHub traffic or visitor-visibility questions. The generic GitHub
repository-info path only handles concrete repository URLs or slugs plus fields
such as stars, language, README, or last commit. With no matching local rule,
dispatch reached the unknown handler.

## Fix

Added seed meanings for:

- `github_repository_platform`
- `repository_reference`
- `github_repository_traffic_signal`
- `github_repository_traffic_question`

The Rust solver and browser worker now compose those roles to route the class to
`github_repository_traffic`. The response uses localized seed templates and
records official GitHub documentation links for the repository traffic UI and
REST traffic metrics. The answer states that GitHub exposes aggregate traffic
such as views, unique visitors, clones, referrers, and popular content to users
with repository access, while individual visitor identities are not exposed.

## Verification

- Added Rust regression coverage in
  `tests/unit/specification/github_repository_traffic.rs`.
- Added a browser-worker Playwright regression in
  `tests/e2e/tests/issue-497.spec.js`.
- Captured before/after CLI repro logs under `.codex-work/issue-497/`.
- Verified `node --check src/web/formal_ai_worker.js`.
