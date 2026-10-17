## Issue #1182 Duplication gates and maintained-parser adoption

| Requirement | Status | Evidence / remaining work |
|---|---|---|
| R8 In-repo duplicate-function gate | Pending strict acceptance | Historical lexical/baseline script is insufficient. The owner's 2026-10-17 strict AST policy is authoritative; see issue-1182-strict-source-duplication.md. Hosted scanner controls and complete zero-independent-clone acceptance remain required. |
| Historical baseline proposal | Superseded by strict policy | Preserve historical inventories as evidence only. Do not initialize or accept a baseline: all independent candidates and coverage gaps must be resolved before strict acceptance. |
| R9 Cross-organization scan and dependency owner selection | Drafted | Isolated org/repo checkout names; Cargo/npm dependency detection; local fixture path; stable marker; refuse issue creation when duplicate search fails. |
| Weekly workflow | Drafted scan-only default | Resolves credential, installs toolchain/runner, scans and uploads record. Opening issues requires explicit workflow input; no issues opened during implementation. |
| R10 Replace local duplicate logic with maintained dependencies | Partial / parent integration | Existing upstream plans and historical groups retained. This script supplies candidate inventory; it does not complete every consolidation or dependency adoption. |
| R11 Links Notation maintained-parser adapter | Drafted against installed 0.16.1 | Uses public parser::parse_document to preserve indentation; crate parse_lino flattens relation paths. No Cargo update required. |
| Full corpus conformance and upstream gap filing | Pending measurement | Exhaustive audit explicitly ignored until separately run in CI and measured gaps tracked. Known dialect risks: inline comments, nested groups, historical escapes and multi-value quoted tails. |
| Delete local parser only after full conformance | Pending | Existing seed loaders keep local parser; no parser deletion or silent fallback. |

Named-function scanning is lexical, not AST based. Rust/JavaScript closures, methods without keywords, regex literals and template interpolation need meta-language grammar integration. No Rust builds, tests, package commands or issue creation were run locally.
