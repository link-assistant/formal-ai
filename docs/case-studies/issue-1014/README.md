# Issue #1014 — fail-closed CI without misleading noise

Issue: <https://github.com/link-assistant/formal-ai/issues/1014>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1015>

## 1. Collected data

The canonical [evidence archive](../../../dev/log/issues/1014/pulls/1015/README.md)
contains all twelve issue-listed default-branch logs, all five initial PR logs,
run/job/artifact/check metadata, 66 annotation responses, issue and PR
discussion surfaces, dependency audits, upstream source/package research, full
template tree metadata, test output, and exact upstream report bodies.

## 2. Timeline

The baseline pipeline began its noisy installs and three macOS core lanes near
12:35 UTC. The slices independently compiled and passed in 999–1,117 seconds;
Gemini warned at 12:45; Auto Release failed at 13:08 because the release cycle
had no fully attributed Formal AI PR. Issue #1014 opened at 13:41 and draft PR
#1015 at 13:43. The timestamped event/evidence table is in the archive.

## 3. Requirements

R1014-1 through R1014-9 cover complete evidence collection; diagnostic
classification without suppression; safe automatic/manual release semantics;
single-build macOS sharding; all-lock JavaScript auditing; removal of harness,
lifecycle, and evidence noise; complete template/Hive Mind comparison and
upstream reporting; tests-first proof; and delivery through PR #1015.

## 4. Root causes

The only failed workflow combined a valid release-integrity decision with an
invalid automatic control flow: the immutable default branch could not create
evidence retroactively, so hard-failing every automatic release deadlocked the
pipeline. Pipeline Status itself was correct. The automatic path now defers
publication while the manual release command remains a hard gate.

The macOS lanes partitioned test execution only after independently compiling
the same graph. JavaScript installs printed advisories but did not enforce an
exit status, and two other npm locks plus the root Bun lock were unaudited false
negatives; a second Bun lock was clean but also ungated. The Gemini harness
scanned its mutable home and under-declared its terminal capabilities. Exact
manifests inside archived evidence were mistaken for live Dependency Graph
projects. Bun's default lifecycle blocking was safe for routine packages, but
OpenCode's pinned package requires its own postinstall to select a Linux
binary; disabling it caused a fresh-CI `Exec format error`.

## 5. Research and prior art

Official cargo-nextest archive and partition support provides compile-once,
run-many slices. Native Bun/npm audits make current locks fail closed while
GitHub Dependency Review continues to inspect PR deltas. Gemini v0.55.1 source
identified the project-lock tree-walk race and its exact true-color check.
Registry research identified Puppeteer 25 as the maintained path beyond the
vulnerable `extract-zip` chain, and uncovered missing musl packages in
Kreuzberg 3.7.2 metadata.

Fresh packaging also exposed an interaction hidden by source-only tests:
Playwright retains optional imports to private chromium-bidi paths and its
maintainers do not support bundling it. It must remain an external production
package in the VSIX. web-capture's browser-commander 0.8 dependency also drops
the packaged browser's `executablePath`; version 0.10 is the minimum forwarding
release.

Complete current Rust, JS, and Python template trees were compared at immutable
heads. All three had change-time Dependency Review but no ecosystem-native
scheduled/current-lock audit, so reports were filed as Rust template #132, JS
template #134, and Python template #58. Gemini #28826, web-capture #153/#154,
and html-to-markdown #459 contain the other reproducible upstream defects;
Playwright's existing #33031 documents its bundling limitation.

The first fresh archive run also confirmed nextest's documented relocation
boundary: compile-time `CARGO_BIN_EXE_*` paths still name the build workspace,
while default archive extraction uses a random temporary directory. A minimal
experiment fails in that default mode and passes when the archive is extracted
into the checked-out workspace, which is now how all three consumers run.

## 6. Tests-first reproduction

Seven implementation-facing tests initially failed 7/7 and are retained in
`tests/unit/ci-cd/issue_1014.rs`. They enumerate the release split, workflow
topology, Gemini environment, dependency-evidence basenames, every JavaScript
lock, every global Bun install, and the required evidence/case studies. The
final module adds an explicit test for every R1014 requirement and one composed
whole-task test. A behavioral self-development policy test exercises the new
status API.

## 7. Implemented fix

Auto Release runs a read-only eligibility preflight and exits successfully with
a GitHub notice when publication must wait. A reusable macOS workflow creates
one nextest archive, extracts it at the original workspace target path, and
feeds three bounded consumers. A new web-stage gate
discovers and audits all five JavaScript locks; dependencies and bundles are
updated until every audit is clean, and routine installs no longer duplicate
audit summaries.

The Gemini E2E uses sibling project/home directories and declares true color.
Global Bun installs explicitly disallow lifecycle scripts except the exact
lockfile-pinned OpenCode package, whose platform-selection script is explicitly
trusted. Twenty-one archived manifests use `.snapshot` names so scanners
cannot discover evidence as a live application.

The real VSIX graph is now a package test: Playwright stays external and its
two production packages are included in the artifact. browser-commander 0.10
forwards the bundled Chromium path. A retained experiment opens a local page
through the extracted 180.94 MB VSIX, not merely checkout files.

## 8. Verification

The retained local evidence includes red and green issue suites, a real
deferred release preflight, five zero-advisory audits, 140 desktop tests, 51 VS
Code tests (now 52 with the real dependency-graph bundle check), an extracted
VSIX browser capture, a trusted pinned OpenCode install, the web build,
actionlint, strict all-feature Clippy, all examples,
2,804 all-feature tests (plus integration binaries), and doc tests. Fresh GitHub
run IDs are recorded in the canonical archive only after they run on the final
PR #1015 implementation head.
