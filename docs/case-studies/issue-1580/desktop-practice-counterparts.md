# Desktop practice and counterpart references

The current Rust practice ledger named six nonexistent Formal AI counterpart
paths. Its new maintained regression scans every declared counterpart across all
74 practices, reports all six original failures, and passes unchanged after the
paths point to their actual owners. Unmapped practices remain open.

| Practice | Original reference | Current owner |
| --- | --- | --- |
| Crate package size | scripts/check-crate-size.rs | scripts/check-crate-package-size.rs |
| Desktop resolution | scripts/resolve-desktop-release.mjs | scripts/desktop-release-resolve.sh |
| Dependency updater | .github/workflows/dependencies.yml | .github/workflows/dependencies-latest.yml |
| CI observations | .github/workflows/ci.yml | .github/workflows/layered-ci.yml |
| Image publication | .github/workflows/docker-publish.yml | .github/workflows/release-staged.yml |
| HTTP recheck | scripts/recheck-links.mjs | scripts/recheck-broken-links.mjs |

The original test draft wrongly required every practice to declare a counterpart.
That assumption is removed before the six-path reproduction; its failure is
retained. Original historical template references remain unchanged.

Practice 27 compares both complete pinned template contracts at commit
`309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb` with the current resolver, manifest,
download adapter and their real consumers. Preserve authenticated producer
identity, matching source-bound tags, matrix-derived complete assets and durable
byte/checksum/protocol/signing observations. Opt-in PR builds from the template
conflict with this repository's main-only production policy and are not copied.

Three maintained owners record 62 passes and one explicit PowerShell skip.
They execute actual resolver, manifest and archive scripts with controlled
providers and fixture binaries. The operation-view owner initially refuses
inherited `NODE_OPTIONS`, as designed. It passes with that environment variable
absent; the 192 MiB heap option moves to the Node command line. The production
guard remains unchanged. This does not qualify live authentication, publication,
native compilation or installed product behavior.

Fresh primary registry responses still show mandatory Chrono in published
command-stream 1.13.0 and link-calculator 0.24.0. Command-stream PR 240 is merged
but unpublished; calculator issue 265 and CodeQL issue 23198 remain open.
Current meta-language is already 1.0.0; its empty relevant dependency selection
does not qualify the historical experimental 0.58.1 graph.

The [complete receipt](../../../experiments/issue-1580/evidence/checks/desktop-practice27-counterparts-20261123/manifest.json)
round-trip verifies 67 members, including original failures, complete streams,
source snapshots, both pinned contracts and fresh dependency responses.
All wider acceptance remains unfinished in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
Supplied commands and edits receive zero independent authorship/reasoning credit.
