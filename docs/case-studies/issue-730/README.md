# Issue 730: complete CI/CD diagnostic and release audit

## Executive result

The audit found one deterministic release failure, one CI false negative, four repository-controlled warning classes, four reviewed third-party warning classes, and one transient E2E flake. The desktop failure was not stale or intermittent: both Windows matrix jobs passed an LF-only checksum file to the released `actions/attest@v4`, whose bundled parser still splits with `os.EOL`. On Windows the complete manifest became one subject name and GitHub rejected it for exceeding 256 characters.

The upstream newline fix in `actions/attest` PR 443 merged on 2026-07-09, but v4.1.1 was released on 2026-06-26. The v4 tag still contained `checksums.split(os.EOL)` when this audit ran. This corrects issue 717's mistaken conclusion that the merged fix was already shipped. The selected fix attests release artifacts by direct path, so provenance no longer depends on checksum-text parsing on any operating system.

The repository changes also:

1. make rustdoc warnings fail CI and repair every warning exposed by that gate;
2. give both workflows a read-only permission default, declare the write permissions each publishing job needs, and bound every desktop job with a timeout;
3. classify exact, reviewed npm deprecations as linked notices while failing on any new dependency stderr;
4. narrowly suppress two warnings emitted inside third-party GitHub actions and link their upstream reports instead of muting Node warnings globally;
5. prevent file-writing Agent CLI requests that mention an existing issue from being routed to `gh issue create`; and
6. repair release-generated changelog whitespace and add the omitted issue 716 fragment-to-release provenance row so the reconstruction guard passes again; and
7. preserve complete run logs, template trees, diffs, API responses, pre-fix reproductions, and Agent CLI streams in this directory.

## Scope and evidence

The audit read issue 730 and all issue comments, and queried every PR 731 feedback endpoint: conversation comments, inline review comments, and reviews. Their API snapshots are under `raw-data/`. It inspected the complete CI/CD tree in this repository and cloned the complete trees of:

- `link-foundation/js-ai-driven-development-pipeline-template` at `727f22a0cfcccb401b3c99812807c515bcba9e8e`;
- `link-foundation/rust-ai-driven-development-pipeline-template` at `bd217bf40c8ce3bd9974b855b6cae84caa006a11`;
- `link-foundation/python-ai-driven-development-pipeline-template` at `c484a816fb2e2e653c38151674a668e3b13a4b7e`; and
- `link-foundation/csharp-ai-driven-development-pipeline-template` at `c6ea17b108f1f0add7a1df615c0192ce16c2e607`.

The trees, revisions, and full `.github` and `scripts` diffs are in `raw-data/templates/`. This repository intentionally has additional release recovery, desktop, VS Code, Pages, multi-registry, and live Agent CLI machinery; differences were evaluated by behavior rather than treated as defects merely because the files are not identical.

The latest relevant failed desktop run was 29465138404 at commit `b5234458`, and the latest complete main pipeline was successful run 29463393707. Both full logs, run metadata, and diagnostics are under `raw-data/ci-logs/`. Timestamps and head SHAs are retained in `desktop-release-runs.json` and `main-release-runs.json`, which prevents a stale green run from being mistaken for validation of a later commit.

## Timeline

| UTC date | Event and significance |
| --- | --- |
| 2026-07-04 | actions/attest issue 440 reported LF checksum parsing failure on Windows. |
| 2026-07-09 | actions/attest PR 443 merged the `/\r?\n/` parser fix after the latest release. |
| 2026-07-15 | PR 729 upgraded Formal AI to `actions/attest@v4`, assuming the merged fix was included in that major tag. |
| 2026-07-16 01:10 | Main run 29463393707 passed, while emitting 61 rustdoc warnings, dependency/action warnings, four file-size warnings, and one retried Pages E2E failure. |
| 2026-07-16 01:46 | Main run 29463393707 completed successfully; its desktop trigger later exercised released v4 code rather than the merged unreleased parser. |
| 2026-07-16 01:50 | Desktop run 29465138404 started; both Windows architectures later failed during artifact attestation with the oversized combined subject name. Linux and macOS jobs passed. |
| 2026-07-16 | Issue 730 reproduced the rustdoc false negative locally, archived upstream source and release metadata, changed provenance to direct paths, and added policy regressions. |

## Diagnostic classification

| Diagnostic | Classification | Root cause and disposition |
| --- | --- | --- |
| Windows `subject name must be 256 characters or less` | True failure | Released `actions/attest@v4` split LF manifests with Windows `os.EOL`. Direct `subject-path` globs remove the parser dependency. |
| 61 rustdoc broken/intra-doc link warnings | CI false negative / true warnings | `RUSTFLAGS=-Dwarnings` does not configure rustdoc and the workflow explicitly cleared `RUSTDOCFLAGS`. The gate is now `RUSTDOCFLAGS=-D warnings`; all links were repaired. |
| Changelog reconstruction guard on released `main` | CI false negative / generated-data drift | Release 0.295.2 left one extra blank line, omitted the final newline, and did not add issue 716 to the fragment-release map. Regenerating the two tracked outputs makes `--check` deterministic again. |
| `download-artifact@v8` DEP0005 | Third-party warning | Exact warning disabled only for that action step; upstream actions/download-artifact issue 484 records the defect. |
| `deploy-pages@v5` DEP0040 | Third-party warning | Exact warning disabled only for that action step; upstream actions/deploy-pages issue 434 records the defect. |
| npm deprecations in desktop and VSIX installs | Third-party warning | Transitive electron-builder/vsce dependencies. Exact messages become linked notices; unexpected stderr remains fatal. Upstream issues 10016 and 1290 track them. |
| Bun `incorrect peer dependency "solid-js@1.9.14"` | Third-party warning | Agent 0.25.0 permits `^1.9.10`, while resolved `@opentui/solid@0.1.107` requires exactly 1.9.12. Preserved and reported as Agent issue 280; not hidden. |
| Docker `update-alternatives` missing `nodejs.1.gz` | Third-party package warning | Ubuntu's Node package registers a manual-page alternative while the minimal image omits that manual. Installing documentation solely to silence a build notice would enlarge the release image without changing runtime correctness. |
| Four 900–999-line Rust file warnings | True preventive warnings | The changed-file warning policy correctly identified `solver_handlers/mod.rs`, `agentic_coding/planner.rs`, `engine.rs`, and `protocol.rs`. All remain below the hard limit; unrelated module splits were rejected as higher-risk scope. |
| Wikipedia Russian-typo E2E first-attempt timeout | Transient flake, not a false positive | The test is network-mocked and the identical Playwright retry passed; 175 tests passed overall. Existing retry surfaced the flake without hiding a persistent failure. No timeout was increased. |
| macOS ad-hoc signing notice | Expected success notice | Absence of release signing secrets intentionally selects the validated ad-hoc path. It was already correctly downgraded by issue 717. |

No current compiler, Clippy, unit-test, packaging, Linux, or macOS release error was hidden in the inspected runs. Cancelled or superseded historical runs are retained in the run lists but are not evidence of a latent defect at the audited SHA.

## Root causes and alternatives

### Released attestation parser versus merged source

`raw-data/upstream/actions-attest-v4-subject.ts` is the source at the live v4 tag and contains `checksums.split(os.EOL)`. `actions-attest-pr-443-subject.ts` is the merged correction and contains `/\r?\n/`. Release metadata in `actions-attest-latest-release.json` establishes the ordering. A merged PR is not evidence that a moving major tag already contains it.

Alternatives considered:

- Convert checksum files to CRLF on Windows. Rejected because published checksum artifacts should remain portable, and provenance would still depend on host-specific text parsing.
- Pin the action to the unreleased merged commit. Rejected because it consumes code outside an upstream release and retains an unnecessary parser in the critical publishing path.
- Wait for a new v4 release. Rejected because current releases would remain broken.
- Pass the actual release artifacts through `subject-path`. Selected because GitHub documents this input directly, the paths are already authoritative in the upload steps, and all platforms take the same code path.

The checksum fragments and consolidated `SHA256SUMS.txt` remain release assets for human and tool verification. They are no longer used as an executable selector for provenance subjects.

### Warning policy

Global warning suppression was rejected. The npm wrapper captures stderr, recognizes only byte-stable prefixes already observed in the archived logs, emits linked GitHub notices, and exits nonzero on every other line. `NODE_OPTIONS` is scoped to the two affected action steps and selects a single Node deprecation code. This keeps new warnings actionable.

The Rust compiler and rustdoc are separate drivers. Setting `RUSTFLAGS` alone cannot make documentation warnings fatal, so the documentation job now owns its explicit `RUSTDOCFLAGS`. `rustdoc-before.log` is the failing reproduction and `rustdoc-after.log` is the passing rerun.

### Agent CLI routing

The required high-level `examples/self-coding/run.sh --live` entry point failed because solve 2.6.0 rejected `formal-ai` as a model alias; the complete output and automated issue comment are retained. Work continued through the repository's release binary, its OpenAI-compatible local server, and the real Agent CLI.

The first direct request mentioned issue 730 while asking for file and workflow edits. The symbolic report router saw a report-action token and an issue token anywhere in the prompt and emitted `gh issue create`, creating duplicate issues 732–734 during progressively smaller reproductions. They were immediately closed and archived under `raw-data/upstream/`. The fix requires the report action and subject to form one local phrase, with stricter word order for verbs shared with file-authoring language. The vocabulary still comes from seed roles; no issue-specific prompt is embedded in production code.

The pre-fix and intermediate streams are `agent-cli-first-attempt.jsonl`, `agent-cli-routing-retry.jsonl`, and `agent-cli-create-file-regression.jsonl`. After the routing fix, a freshly rebuilt Formal AI release server and Agent CLI 0.25.0 authored `agent-authored-finding.md`, verified it through the shell, exited successfully, and created no new issue. The complete successful stream, stderr, and generated plan are `agent-cli-final.jsonl`, `agent-cli-final.stderr.log`, and `agent-cli-final-plan.lino`. A regression pins the authored bytes and the no-issue outcome.

## Template comparison

| Area | Template comparison | Formal AI result |
| --- | --- | --- |
| Workflow permission baseline | Templates vary and mostly rely on workflow/job declarations. | Both Formal AI workflows now default to `contents: read`; publisher jobs elevate explicitly. |
| Job timeouts | Requested templates bound more build/test jobs than the desktop workflow did. | Every desktop job now has a 10–30 minute bound. |
| Warning gates | Rust template uses strong compiler/lint gates but does not expose Formal AI's project-specific doc command. | Formal AI adds an explicit rustdoc gate and repairs its complete warning set. |
| Dependency installation | Template package ecosystems differ; none can replace Electron/vsce-specific policy. | A shared fail-closed installer covers both desktop and VSIX steps. |
| Release attestation | Formal AI's desktop/extension release topology is project-specific. | Both attestation sites use direct artifact paths and are asserted together. |
| Change detection and file-size policy | Issue 717 already imported the relevant complete-range/changed-warning behavior. | Existing coverage remains; the four current warnings are correctly classified, not suppressed. |
| Action pinning | Formal AI and the requested templates use moving major action tags. | Full-SHA pinning is a broader shared supply-chain policy, not the cause of this incident; this PR does not silently mix update strategies. |

No identical unreported defect was found in a requested template. The repository-specific third-party warnings were reported to their owning projects rather than to unrelated pipeline-template repositories.

## Upstream research and reports

Primary references used in the investigation:

- GitHub artifact-attestation documentation: https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations
- GitHub workflow permissions and timeout syntax: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
- GitHub least-privilege `GITHUB_TOKEN` guidance: https://docs.github.com/en/actions/tutorials/authenticate-with-github_token
- GitHub action full-SHA security guidance: https://docs.github.com/en/actions/reference/security/secure-use
- rustdoc lint documentation: https://doc.rust-lang.org/beta/rustdoc/lints.html
- rustdoc intra-doc link syntax: https://doc.rust-lang.org/rustdoc/linking-to-items-by-name.html
- Node warning and `NODE_OPTIONS` documentation: https://nodejs.org/api/cli.html

Upstream records:

- Existing root-cause report and merged fix: https://github.com/actions/attest/issues/440 and https://github.com/actions/attest/pull/443
- download-artifact DEP0005: https://github.com/actions/download-artifact/issues/484
- deploy-pages DEP0040: https://github.com/actions/deploy-pages/issues/434
- electron-builder transitive deprecations: https://github.com/electron-userland/electron-builder/issues/10016
- vsce transitive deprecations: https://github.com/microsoft/vscode-vsce/issues/1290
- Agent CLI incompatible Solid peer: https://github.com/link-assistant/agent/issues/280

JSON snapshots under `raw-data/upstream/` make the research independently reviewable if remote issue text changes.

## Reproduction and verification

The minimal pre-fix documentation reproduction was:

```bash
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features
```

It failed on 61 warnings. The same command passes after the link corrections. Focused policy tests assert:

- both provenance calls use direct artifact paths and no checksum input;
- each desktop job has explicit permissions and a timeout;
- both workflow defaults are read-only;
- rustdoc warnings are fatal;
- dependency diagnostics are exact and fail closed;
- action warning suppression is code-specific and step-scoped; and
- existing-issue file-writing prompts cannot produce `gh issue create`.

The local pre-fix and post-fix logs are stored at the root of this case study. The full all-features run passed 100 integration tests and hit four parallel loopback tests with the same `WouldBlock` resource-contention error; all four passed immediately with `--test-threads=1`, so no production timeout was changed. The captured suite and focused-rerun logs preserve that classification.

### Fresh branch verification

CI/CD Pipeline run [29482019343](https://github.com/link-assistant/formal-ai/actions/runs/29482019343) was created at 2026-07-16 08:02:34 UTC for implementation commit `13379cc32500e1b4fe786f16187baaa677096f0d` and completed successfully at 08:20:17 UTC. Every required pull-request job passed, including the new real Agent CLI existing-issue regression, strict lint and rustdoc gates, the complete Rust test suite, local-demo browser tests, coverage, changelog checks, and package construction. Release-only jobs were correctly skipped for the pull-request event.

The run produced no error annotations or failed steps. Its four annotations were the two reviewed notices for Agent CLI compatibility diagnostics and retained LCOV without a Codecov token, plus preventive warnings for `src/seed.rs` at 909 lines and `src/dreaming.rs` at 903 lines; both files remain below the 1,000-line hard limit. The complete 21,226-line log, run metadata, and job/step metadata are `raw-data/ci-logs/branch-29482019343.log`, `branch-29482019343.json`, and `branch-29482019343-jobs.json`. A second run on the evidence-only follow-up commit is required before the pull request is marked ready, so remote verification cannot be invalidated by the act of committing this evidence.

## Requirements traceability

| Issue requirement | Evidence/result |
| --- | --- |
| Read issue and all comments | `raw-data/issue*.json` and all three PR feedback API snapshots. |
| Recheck false positives, false negatives, warnings, and errors | Complete classification table and both full workflow logs. |
| Compare complete JS, Rust, Python, and C# template trees | Revisions, inventories, and complete `.github`/`scripts` diffs under `raw-data/templates/`. |
| Preserve data and logs | Full CI logs, diagnostics, API metadata, upstream source, reproductions, and Agent CLI streams in this directory. |
| Deep history, alternatives, libraries, and online research | Timeline, root-cause alternatives, template matrix, primary references, and upstream snapshots above. |
| Report upstream defects | Five new linked reports plus the existing actions/attest issue and fix. |
| Test before fix | Attestation, permission, routing, and rustdoc pre-fix failures are preserved; automated regressions cover each repository-controlled defect. |
| Use Formal AI through Agent CLI | Failed high-level solve run plus the successful post-fix real-CLI run, generated artifact, plan, and streams are retained; the routing defect discovered by self-hosting has regression coverage. |
| Apply fixes throughout codebase | Both workflows, both attestation sites, both npm install sites, all rustdoc warnings, and the shared report router are covered. |
| Prepare a patch release | `changelog.d/20260716_073000_issue_730_ci_audit.md`; the release workflow performs the version bump after merge. |
| One pull request | https://github.com/link-assistant/formal-ai/pull/731 |
