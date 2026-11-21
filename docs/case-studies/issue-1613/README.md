# Hardcoded solutions and source discovery

[Issue #1613](https://github.com/link-assistant/formal-ai/issues/1613) ·
[Pull request #1614](https://github.com/link-assistant/formal-ai/pull/1614) ·
[Hive Mind issue #4122](https://github.com/link-assistant/hive-mind/issues/4122)

## What the alias did

The two rows in the reported `procedural_how_to` table keyed on `какать`
and `покакать` and replaced either task with `хорошо покакать`. They were
an exact prompt-to-title mapping. The consumers applied them before building
the MediaWiki endpoint and before constructing the broader search query. The
walk could still verify an article it retrieved, but its choice of article
for these two tasks came from the installed table. The change was committed
while resolving the Codex web-tool interaction from issue #1396. Earlier
changes had already implemented localized wikiHow URLs, an archive fallback,
and the reusable bounded source walk.
[Reported rows](https://github.com/link-assistant/formal-ai/blob/f574289f4559f1d8b801eca9d60bc087a4cd888f/data/seed/handler-rules.lino#L455-L461) ·
[Introducing commit](https://github.com/link-assistant/formal-ai/commit/74b49af6e35875c3a8708d24703a67ef3c443d22) ·
[PR #1398](https://github.com/link-assistant/formal-ai/pull/1398) ·
[PR #1608](https://github.com/link-assistant/formal-ai/pull/1608) ·
[Issue #991](https://github.com/link-assistant/formal-ai/issues/991)

It took two controlled replays to distinguish the shortcut from the missing
fallback. In the original short-question case, only the query with the
unchanged task would yield the candidate. The old code hit the preset article
and the archive without issuing that query. In a held-out Russian case,
`читать вслух`, the exact URL was missing and the only acceptable article
title was returned by search. The original revision produced no instructions.
[Before-fix test log](evidence/verification/baseline-procedural.log)

## Timeline on 2026-11-21 in UTC

| Time | Event and record |
| --- | --- |
| 11:18:04 | The alias is committed on the #1608 branch. [Commit](https://github.com/link-assistant/formal-ai/commit/74b49af6e35875c3a8708d24703a67ef3c443d22) |
| 16:07:26 | The issue is opened in Hive Mind and triggers an attempt there. [Issue snapshot](evidence/github/issue.json) |
| 16:09:08 | Hive Mind's attempt exits on `manifest unknown`; the full log records the failed pull at lines 466–467. [Run 44922819938](https://github.com/link-assistant/hive-mind/actions/runs/44922819938) |
| 16:10:57 | The issue is renamed. [Timeline](evidence/github/issues-1613-timeline.json) |
| 16:12:28 | The issue is transferred to Formal AI. The other Hive Mind run records a transfer-only health check. [Run 44923240978](https://github.com/link-assistant/hive-mind/actions/runs/44923240978) |
| 16:16:51–53 | The Formal AI draft run selects Formal AI `f574289f…`, Hive Mind `3e9dcff2…`, and image `konard/hive-mind:2.45.1`. At lines 1782–1783 it records `[DRAFT_IMAGE_UNAVAILABLE]` after one pull attempt. [Run 44923287323](https://github.com/link-assistant/formal-ai/actions/runs/44923287323) · [Attempt identity](evidence/session/formal-ai-attempt-identity.json) |
| 16:16:55 | The bot posts `serving-identity-unverified` and zero capability credit. No model or tool exchange was observed in that attempt. [Issue comment](https://github.com/link-assistant/formal-ai/issues/1613#issuecomment-7065525709) |
| 16:24:21 | GitHub publishes the Hive Mind v2.45.1 release. The same image tag resolved when inspected later. [Release](https://github.com/link-assistant/hive-mind/releases/tag/v2.45.1) · [Registry response](evidence/hive-mind/hive-image-manifest.json) |
| 19:27:26 | PR #1614 is cross-referenced by GitHub. [Initial draft](evidence/github/initial-pr.json) |
| 19:38:46 | The image-selection gap is reported upstream with a reproduce-by-mock script, workflow references, and recovery options. [Hive Mind #4122](https://github.com/link-assistant/hive-mind/issues/4122) |

The registry error happened before the draft process started. The capture
contained 551 console bytes and the command completed with exit code 1 in about
1.4 seconds. This establishes why the workflow could not verify serving identity.
The selected tag being absent during both attempts and resolving later supports
a publication race. GitHub's release timestamp alone cannot date the publication
of the Docker manifest. The local mock reproduced the classifier independently:
the first pull returned `manifest unknown`, the second would have succeeded,
and the implementation made only the first call.
[Attempt outcome](evidence/session/attempt-outcome.json) ·
[Upstream reproducer](evidence/hive-mind/hive-report.md)

## Implementation and alternatives

- Use the original task with the localized publisher. An exact title is a cheap
  first probe. On a missing page or an unreadable MediaWiki error, follow the
  publisher's search results inside the existing source-walk limits. Validate
  candidate titles against all topic words and preserve Russian negation.
  Productive Russian infinitive aspect variation is shared by the JS and Rust
  candidate checks. Accept an article only after extracting genuine steps and
  record the capture identity with every step. A browser with a configured
  capture component can also look up a publisher title after direct transport
  fails, then verify the article through the existing HTML and archive checks.
  MediaWiki's search API supplies candidate titles and its parse API supplies
  the page body. The application's existing Stack Exchange path and its
  corroboration thresholds remain independent.
  [MediaWiki search](https://www.mediawiki.org/wiki/API:Search) ·
  [MediaWiki parse](https://www.mediawiki.org/wiki/API:Parsing_wikitext)
- Direct search and article parsing have separate failure modes. Searching
  arbitrary public web snippets could improve recall, but a high-ranked
  fragment does not establish a matching procedure or its provenance. Reusing
  the publisher channel and verifying the captured article keeps that boundary.
  Generic stemming could improve recall, especially with inflection, but
  Snowball's Russian stemmer removes suffixes and does not itself resolve an
  aspect prefix. The current matching is deliberately bounded, so a synonym or
  a procedure split over multiple related tasks can still need a stronger
  morphological or semantic discovery method.
  [MediaWiki search and discovery](https://www.mediawiki.org/wiki/API:Search_and_discovery) ·
  [Snowball Russian stemmer](https://snowballstem.org/algorithms/russian/stemmer.html)
- For captured GitHub issues, delete the production URL-to-document table and
  its 139 Rust mirrors, and give the deterministic test a source reader that
  explicitly uses the benchmark capture. For the CLI, use GitHub's public
  issue API through the bounded native source client under the existing network
  policy. Rebuilding a slim image with the same binary cannot remove a capture
  compiled into that binary. Changing the source of the bytes works on both
  binary distribution paths.
  [GitHub get-an-issue API](https://docs.github.com/en/rest/issues/issues#get-an-issue)
- For translation replay, remove compile-time enumeration of the optional
  API-cache directory. The same test can supply saved HTTP responses to an
  explicit transport, and the production cache can fill as it retrieves
  sources. Running the standalone build script with a legacy fixture tree
  still generated the owned-source manifest and left out the replay bundle.
  [Build regression](evidence/verification/build-script-optional-inputs-round2.log) ·
  [Build-script probe](evidence/verification/translation-build-probe.log)
- For the three encyclopedia responses, preserve the original bytes, capture
  times, and digests in the test source cache. The production clients retrieve
  or replay their own cached copies. The browser baseline returned a capture
  from a fresh offline worker without a transport. The new replay first
  observes the offline miss, acquires the capture via a supplied transport,
  and then verifies the worker's cache hit.
  [Baseline replay](evidence/verification/baseline-encyclopedia-precache.log) ·
  [Browser verification](evidence/verification/source-qualified-definition.log)
- For the draft image race, gate the run on the exact digest or wait for the
  selected tag within a bounded retry window. Preserve the checkout/image
  identity proof. The report includes manually checking the exact manifest
  and rerunning the affected workflow after publication, with a separate
  classification for an unavailable tag.
  [Docker Registry manifest API](https://docs.docker.com/reference/api/registry/latest/operations/HeadImageManifest/) ·
  [Hive Mind #4122](https://github.com/link-assistant/hive-mind/issues/4122)

## Acceptance and evidence

| Requirement from the issue | Evidence and current coverage |
| --- | --- |
| Explain and remove the reported hardcoding | Reproduced the exact bypass, removed both aliases, and added original-task and held-out replays. |
| Generalize how-to across runtimes | Both runtimes use localized discovery, matching, and bounded article reads. The browser replay covers direct, archive, capture component, missing evidence, unrelated results, negative titles, and opt-out cases. The first native PR run passed all four localized Chat and Responses replays. Tasks outside the supported publisher contract still need broader verification. |
| Keep canned input and expected output in tests | The two replay topics and their expected articles are explicit in test code. The benchmark issue test injects its recorded input. Additional production solution tables remain in the [repository audit](repository-audit.md). |
| Use recoverable snapshots only to speed tests. Exclude pre-cache from lightweight distributions | The crate mirror no longer includes the 139 issue bodies. The previous native release returned the stored body with an empty cache, which the [baseline test](evidence/verification/baseline-native-issue-corpus.log) rejects. CI can replay the same command first with an empty cache and then with a supplied capture. Remaining embedded and web-distributed captures are listed in the [repository audit](repository-audit.md). |
| Align the vision and conflicting guidance | [VISION](../../../VISION.md) and the [natural language design](../../design/no-hardcoded-natural-language.md) distinguish vocabulary, rendering templates, live discovery, and fixed test inputs. |
| Collect logs, reconstruct sequence, evaluate existing approaches, and diagnose the root causes | Original artifacts, complete available workflow logs, issue timeline, and focused tests are indexed below. The alias and the independent image pull failure are both reproducible from the captured evidence. |
| Add diagnostic output when root cause cannot be established | The existing pull error, capture manifests, and original-task requests identify the causes. The source-walk trace records failed requests and whether recovered steps came from a live or archived capture. |
| Report reproducible upstream issues with workarounds and fixes | The reproducible Hive Mind failure is in [#4122](https://github.com/link-assistant/hive-mind/issues/4122). The [publisher probes](evidence/wikihow/status.txt) from this environment returned an HTTP 500 with no body. That observation alone did not isolate a publisher bug. Previous issue #1396 records a successful parse and an archive rate limit from separate probes. |
| Complete all changes in one PR | #1614 is the single implementation branch. The issue's [sub-issue listing](evidence/github/issues-1613-sub_issues.json) is empty. The [vision audit](repository-audit.md) covers related broader work. |

Two additional cases showed topic substitution. An unnamed algorithm request
selected a Python sorting answer, and an unknown grammatical-number request
selected tomato lexemes. An unnamed operation now gets a clarification, and
general research receives the requested word when the document recipe has no
matching concept. The [algorithm baseline](evidence/verification/baseline-unnamed-algorithm.log)
and the [meaning baseline](evidence/verification/baseline-meaning-detail.log)
reproduce those choices.

The first Layered CI run at `fdf49eb07` found a test reading the deleted Rust
mirror of issue #698 and a stale runtime inventory. The test now reads the
benchmark capture explicitly and the inventory was regenerated from the
current source. Both suites passed locally after these updates.
[Layered CI run](https://github.com/link-assistant/formal-ai/actions/runs/44943549869) ·
[Issue replay](evidence/verification/main-only-planner-twins-round2.log) ·
[Runtime inventory](evidence/verification/runtime-feature-inventory-round2.log)

The first native run passed the four procedure replays, including the
short prompt and the Responses route. The issue regression received the
cache-miss message with the expected API endpoint. Its assertion used the word
`offline`, which the rendered message omits. The assertion now compares the
actual message and still requires the supplied issue body on the next call.
[Native CI run](https://github.com/link-assistant/formal-ai/actions/runs/44943550052) ·
[Full workflow log](evidence/workflows/pipeline-44943550052.log.gz)

## Evidence archive

- [GitHub snapshots](evidence/github/) include the issue, all discussion,
  timeline, sub-issue listing, initial PR, inline comments, and reviews.
- [Draft workflow logs](evidence/workflows/) contain all available frames from
  the three related GitHub Actions runs, compressed losslessly. The [draft
  session](evidence/session/), [main job](evidence/main-evidence/), and [build
  identity claim](evidence/build-claim/) are the public diagnostic artifacts
  [14714562300](https://github.com/link-assistant/formal-ai/actions/runs/44923287323/artifacts/14714562300),
  [14714806089](https://github.com/link-assistant/formal-ai/actions/runs/44923287323/artifacts/14714806089),
  and [14714621720](https://github.com/link-assistant/formal-ai/actions/runs/44923287323/artifacts/14714621720).
  The session manifest lists `build.capture.jsonl` and `build-outcome.json` as
  missing because the launched draft used a shared build. The compiled binary
  and its digest are referenced by the identity claims.
- [The upstream report and registry evidence](evidence/hive-mind/) record the
  workaround, reproduced pull classification, release timestamp, and the
  registry manifest observed later.
- [Verification logs](evidence/verification/) record the regressions before
  and after the fix and adjacent offline, practical-guidance, and procedural
  checks. The native cases in the server run were skipped because that run
  lacked a source-built native binary.
- [Local Formal AI development attempt](evidence/formal-development/) holds a
  request, endpoint identity, Agent CLI transcript, and server events. The
  observed model tool actions repeated a search for the draft placeholder and
  did not produce an independent analysis or implementation.
- [SHA256SUMS](evidence/SHA256SUMS) covers the archived files. Run
  `sha256sum -c SHA256SUMS` from the evidence directory.
