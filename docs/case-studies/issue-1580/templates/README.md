# Complete template inventory and CI review

Source trees were pinned, downloaded completely and verified against every Git blob before comparison. Same-name matches are review aids; they do not establish behavioral equivalence. All file roles, including application source, documentation and historical evidence, remain visible in the TSV inventories.

| Language | Source commit | Blobs | Workflows | CI candidates |
| --- | --- | ---: | ---: | ---: |
| js | [74de898dd6ce](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/tree/74de898dd6ce548af5bcb9359e1dae75fd4ebd75) | 561 | 8 | 233 |
| rust | [309387f761f9](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/tree/309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb) | 373 | 8 | 124 |
| python | [c532c173801d](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/tree/c532c173801dfb12e681b0730c4abbf93aefd0ca) | 329 | 6 | 150 |
| php | [3a034b27a560](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/tree/3a034b27a56067971ca74dd4c74c7707056b215d) | 608 | 6 | 78 |
| csharp | [a5b9a1a46686](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/tree/a5b9a1a46686cfe844f9afa2ace030094969e766) | 337 | 6 | 117 |
| java | [dd803a2d9fa5](https://github.com/link-foundation/java-ai-driven-development-pipeline-template/tree/dd803a2d9fa5b3a992b435acce73162274fb9e8b) | 207 | 4 | 33 |
| cpp | [3769fa70a1bc](https://github.com/link-foundation/cpp-ai-driven-development-pipeline-template/tree/3769fa70a1bc30ce9516e01f78bcd7edea3b3f80) | 550 | 7 | 94 |
| go | [65c686846d73](https://github.com/link-foundation/go-ai-driven-development-pipeline-template/tree/65c686846d7327d8c835a0d30b1a76cc6345cfc6) | 42 | 3 | 32 |

## Policy comparison and current decisions

The current Hive Mind `docs/CI-CD-BEST-PRACTICES.md` was read in full. Its recommendations are evaluated against this repository's explicit default-branch build policy; adopting a template does not authorize PR application builds, Docker execution, deployments or weaker verdicts.

| Topic | Formal AI application / review status |
| --- | --- |
| Pinned runners and native architecture | Keep pinned `ubuntu-24.04`; recovered browser runner follows the pinned policy. Native ARM release paths remain; do not introduce QEMU. |
| Change detection and required checks | Documentation-only PRs now emit required contexts without compiling. Unknown/failed detection triggers the check. Never filter production seed data as generic non-production `data/`. |
| Size, lint and formatting | Full local JS twins pass; existing warning inventory remains open. Keep global thresholds. |
| Changelog and publication identity | Patch fragment is the existing release trigger; retain trusted publication/OIDC and pre-publication freshness checks. |
| Merge checks and machine writers | Missing required main contexts are a real failure. Configure only with a verified bypass/PR design for automated ledger writers; compatibility is not yet established. |
| Serialized default-branch writers | Preserve non-cancelling write concurrency, immutable source identity and explicit GH006/GH013 failures; do not retry a protected-branch refusal as a network outage. |
| External secrets and publication preflight | Preserve verified/failed/unknown states. PRs may report unavailable external publication without claiming it succeeded; main must fail with the concrete action required. Token presence alone is not verification. |
| Workflow lint tools | Existing actionlint, ShellCheck, runner labels and security checks remain. Local ShellCheck used the verified v0.11.0 binary; no Rust build was performed. |
| Scheduled dependency checks | Latest dependencies run still exposes real source-size warnings. A successful install is not full runtime validation. |
| Download retries | Local OSV/source download fixes retain finite bounds. The C++ Doxygen archive command lacks all-error retry coverage; bounded real-curl regression and upstream #35 document it. |
| Retry classification and cleanup | Duration collection uses bounded retry and propagates unread/incomplete evidence; invalid credentials, refspecs and immutable 409 conflicts must remain terminal. |
| Input defaults, skip notices and verdict guards | Recovered draft and current context-reporting controls preserve explicit skip/failure states. Successful scope gates are not capability evidence. |
| Real GitHub end-to-end qualification | Keep opt-in live effects and default-off verbose tracing. Docker remains prohibited on PRs; the failing live qualification requires real pull diagnostics. |
| Packaging warnings | Retain extension runtime files; the Playwright external package cannot be removed solely to silence VSIX file-count warnings. Resolution remains open. |

## Review boundaries

`*-files.tsv` covers every blob. `*-ci-files.jsonl` conservatively selects workflows, composite actions, scripts/tests and build/policy metadata. `*-workflows.jsonl` preserves each complete parsed workflow document and Git blob identity, including action inputs, environment/default/service/container fields, outputs and reusable workflow bindings; its smaller job/step fields are only a search index. Three integrity regressions reject dropped execution fields and tampered source blobs. These generated records do not substitute for resolving shell/package-script/configuration references and reviewing their behavior. That semantic/transitive review remains unfinished and must be completed in this PR.

The latest related C++ merged change, PR #34, was reviewed before filing #35; it refreshes dependencies and release configuration but does not repair truncated-transfer retry. The Go pipeline-status helper was read; its failure/cancelled classification is not reported as a bug without evidence. Java already retries the OSV download with `--retry-all-errors`; it is not included in the C++ finding.

Reproduction tooling: `experiments/issue-1580/templates.mjs` selects source identities, `full-template-trees.py` fetches complete archives and verifies every Git blob, and `compare-template-trees.mjs` generates these inventories. Run against the pinned manifest to retain historical reproducibility; selecting current heads intentionally creates a new comparison checkpoint.

## Executed cross-template observations

`rust/tests/web/issue-1580-template-observations.test.mjs` is in the fast PR inventory. Its ten local tests all passed, with no runtime skips. The unmodified Go, Java, Python and PHP detectors all returned exit 0 with negative code-change outputs when the selected Git comparison/repository was unavailable. Code/docs/empty controls separate the defect from legitimate negative observations. Java/Python also fail their first-commit mock, while PHP's legitimate initial-tree fallback succeeds; PHP still misclassifies an entirely unknown repository. A real initial Go Git repository reproduces its false negative. The production Formal AI detector now propagates errors and permits a tree fallback only for an established initial commit.

The Go workflow model also reproduces skipped lint/build/selected release for both manual modes, followed by success from the unchanged terminal helper. This is modeled dependency propagation using the pinned YAML and the real helper, not hosted release execution. A candidate model repair admits both modes and still blocks release after forced lint/test/build failures. Python #18/#19 already fixed its analogous dispatch admission and are not reported again. Java #29 already tracks its missing aggregate status job. Latest merged related changes (Go #16, Java #25, Python #152 and PHP #24) were reviewed before preparing the five distinct upstream reports. Their report bodies, source identities and limitations are preserved in `experiments/issue-1580/upstream/`; filing URLs belong in `filed-issues.txt`.

The five reports are now filed with public executable evidence at commit `71e1dee442fd74589df20a1d6f33710b6dec0d18`: [Go manual dispatch #18](https://github.com/link-foundation/go-ai-driven-development-pipeline-template/issues/18), [Go Git observation #19](https://github.com/link-foundation/go-ai-driven-development-pipeline-template/issues/19), [Java #30](https://github.com/link-foundation/java-ai-driven-development-pipeline-template/issues/30), [Python #156](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/156), and [PHP #26](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/26). They are reports, not claims that the template repositories have applied fixes.

## Static reference review aid

`trace-template-ci.mjs` follows literal local actions, reusable workflows, package commands and source/configuration references from all 48 workflow roots. Its compressed per-language graphs retain commands, reference reasons and unresolved paths. The graphs deliberately include conservative directory/package expansions and can contain irrelevant literal/document references. They are lexical inventories, not complete execution closures or semantic verification. Dynamic/glob paths, command working directories and implicit tool configuration still require source review. Every visited file is checked against the pinned Git blob, and two bounded regressions exercise nested helpers and transitive source tampering. Full semantic review remains unfinished.
