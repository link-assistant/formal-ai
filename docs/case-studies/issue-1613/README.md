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
- For a task outside the registered catalogues, use web search to find candidate
  pages and check the procedure on the captured publisher page. The page or
  section heading must match the task, and distinct ordered procedures under
  the same heading need more specific wording. A captured GitHub Docs page
  exposed a navigation heading above the article and short steps that the
  MediaWiki filters previously discarded. Scope parsing to the main content
  and preserve the publisher's ordered list and icon labels. The source URL
  and digest identify the bytes used in the answer.
  [GitHub Docs publisher page](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-new-repository)
- Replace the embedded recurrence harvest with a repeatable publisher lookup.
  Resolve localized labels through Wikifunctions and, when needed, the matching
  Wikipedia title and its language links. Read the definition, abstract
  implementation, and test objects. Check that the recurrence terminates for
  nonnegative integers before generating Python. The stored harvest still
  exercises the formalization tests from an explicitly supplied path.
  [Wikifunctions API](https://www.wikifunctions.org/wiki/Special:ApiHelp/wikilambdaload_zobjects) ·
  [MediaWiki language links](https://www.mediawiki.org/wiki/API:Langlinks)
- Resolve currency pairs through a dated publisher endpoint and expose the
  effective rate date. The previous dedicated conversion fixed both the rate
  day and the ruble denominator, while a separate calculator path used its
  compiled defaults. Frankfurter exposes provider-scoped CBR quotes for RUB
  and a reference-rate API for other covered pairs. The direct CBR XML
  remains useful for checking the source but is less convenient for
  cross-currency requests from browsers. Six archived pair responses
  verify derivations, including a JPY/CAD question the previous handler
  could not recognize. The offline path preserves exact same-currency
  arithmetic and reports when it lacks a verified quote.
  [Frankfurter API](https://frankfurter.dev/) ·
  [CBR endpoints](https://cbr.ru/development/sxml/) ·
  [ISO 4217 maintenance agency](https://www.six-group.com/en/products-services/financial-information/market-reference-data/data-standards.html?printout=1) ·
  [Archived source evidence](evidence/currency/) ·
  [Calculator #266](https://github.com/link-assistant/calculator/issues/266)
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
| Generalize how-to across runtimes | Both runtimes use localized discovery, matching, and bounded article reads. The browser replay covers direct, archive, capture component, missing evidence, unrelated results, negative titles, and opt-out cases. The indexed publisher replay covers a task outside the catalogues, verifies the original capture, and rejects a page unrelated to the task. |
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

The second native run passed the localized procedure and Agent CLI replays,
including the empty-cache and supplied-capture checks. The two Layered CI
shards found stale governed source hashes after the translation documentation
and source-cache test changed. Regenerating the witness changed ten digest
fields. The inventory still contains 1,185 cases, including the same 221
supported cases and 964 recorded unsupported cases. Security passed on retry
after a RustSec download failure on the first attempt.
[Native CI](https://github.com/link-assistant/formal-ai/actions/runs/44946231524) ·
[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/44946231224) ·
[Security](https://github.com/link-assistant/formal-ai/actions/runs/44946231401) ·
[Witness comparison](evidence/verification/governed-baseline-round3-delta.log)

The third Layered CI run passed on `2b5ae6485`. The Fast Linux check in
the CI/CD pipeline found that the generated role registry omitted `noun`,
which the source policy used to qualify a practical-help goal. The pipeline
log records the difference at lines 753–754. The native type, lint, and HTTP
checks passed. Regenerating the role registry added that entry, and the same
seven source governance checks then passed locally.
[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/44949387617) ·
[CI/CD pipeline](https://github.com/link-assistant/formal-ai/actions/runs/44949387904) ·
[Pipeline log](evidence/workflows/pipeline-44949387904.log.gz) ·
[Local source checks](evidence/verification/round4-fast-governance.log)

The practical-help route contained a second form of the same defect. Its
expected NHS quote also served as its own evidence. The before-fix replay
failed because the solver returned an answer while the controlled transport
recorded zero requests. The new replay changes the page body twice and checks
that each answer quotes the passage observed on that run. The production
policy declares the NHS page, section, and matching cue. The source client's
verified cache can supply a previously retrieved capture. With no matching
capture, the solver asks a localized clarifying question. A browser request
sent with an `Origin` header received HTTP 200 without an `Access-Control-Allow-Origin`
header, so a browser with a capture component can try that transport as well.
[Baseline failure](evidence/verification/baseline-practical-help-current-page.log) ·
[NHS source](https://www.nhs.uk/conditions/constipation/) ·
[Practical-help replays](evidence/verification/practical-help-round3-final.log) ·
[Local gate results](evidence/verification/all-gates-round3-final.log)

The documentation rule used the same answer for every accepted phrasing
because its operand reader recognized only one pandas method. A previously
uncovered request about `pandas.Series.diff` produced no result and made
no publisher requests in baseline replay. The live pandas index lists three
different methods ending in `join`. The replacement checks a qualified
name against the published reference page, consults the publisher's index
to resolve other spellings, and asks the user to select a method when the
publisher lists several candidates. Its response gives a bounded excerpt
and a provenance link from the capture. The replay supplies controlled
pages for methods from two publishers and rejects a page about a different
method. The source text retains its publisher language.
[Baseline replay](evidence/verification/baseline-docs-method.log) ·
[Pandas reference](https://pandas.pydata.org/docs/reference/api/pandas.Series.diff.html) ·
[pandas index](https://pandas.pydata.org/docs/searchindex.js) ·
[Captured index entries and digests](evidence/verification/pandas-publisher-index.log) ·
[NumPy reference](https://numpy.org/doc/stable/reference/generated/numpy.ndarray.reshape.html) ·
[Sphinx documentation](https://www.sphinx-doc.org/en/master/usage/extensions/intersphinx.html)

The fourth native HTTP run exposed an earlier routing conflict. Its
`pandas Series.diff` replay entered the workspace route because the path
reader treats dotted tokens as possible filenames. The dispatcher now
excludes the selector from path evidence when a registered documentation
publisher binds it. The replay checks both dotted spellings before and after
attaching the publisher capture. A request that explicitly asks to read the
file `Series.diff` in the workspace still produces a file action.
[Native CI](https://github.com/link-assistant/formal-ai/actions/runs/44952598777/job/136993775725) ·
[Fourth pipeline log](evidence/workflows/pipeline-44952598777.log.gz) ·
[Fourth Layered CI log](evidence/workflows/layered-44952598227.log.gz)

The initial generalized repair still limited title discovery to wikiHow.
The held-out Wikibooks replay supplied a broader result page that ranked a
bread-making book ahead of the requested recipe. The worker made no scoped
Cookbook search and produced no instructions. Both runtimes now use each
MediaWiki publisher's declared search scope, check the ordered subject and
qualifiers, and extract one ordered procedure from the verified article. The
same path retrieves bread and pancakes independently. The replay also asks
for milk-free pancakes and rejects the recipe with milk. The publisher
responses are explicit test inputs with recorded URLs and digests.
[Baseline replay](evidence/verification/baseline-mediawiki-procedures.log) ·
[Browser replay](evidence/verification/round5-procedural-runtime-parity-final.log) ·
[MediaWiki search API](https://www.mediawiki.org/wiki/API:Search) ·
[Bread recipe](https://en.wikibooks.org/wiki/Cookbook:Basic_Bread_Dough) ·
[Pancake recipe](https://en.wikibooks.org/wiki/Cookbook:Pancakes_(North_American))

The fifth pipeline ran 22 native and browser replays in one step. The pancake
request in the multi-turn web-search case lost its connection after 25.023
seconds, matching the fixture's 25-second child-process cutoff. The first
request against that child and the independent recipe replay passed. The
fixture now uses the enclosing test's lifetime and surfaces the child's exit
status and stderr when a request fails.
[Native job log](evidence/workflows/native-job-137003375836.log.gz) ·
[Fifth pipeline log](evidence/workflows/pipeline-44955707612.log.gz) ·
[Fifth Layered CI log](evidence/workflows/layered-44955707231.log.gz)

## Publisher facts and offline replays

The browser previously loaded the checked-in entity responses into its
runtime cache at startup. The native seed loader turned the same responses
into installed answers. A country question in an otherwise fresh runtime
could therefore succeed without acquiring publisher evidence. Retrieval now
binds the requested subject and property, checks exact labels and aliases,
then verifies the selected claim and value. The browser and native tests
attach the historic captures explicitly. An empty cache with networking
disabled gives no publisher fact.
[Subject and source replays](evidence/verification/publisher-fact-subject-gate.log) ·
[Language and cache matrix](evidence/verification/publisher-fact-matrix.log)

A live search for Portugal returned a country, a surname, a national football
team, a constituency, and a language. The country was the only matching
candidate with a capital statement. It carried a preferred claim for Lisbon
and a normal claim for Coimbra with an end date in 1255. This is a case where
checking the exact subject and respecting the publisher's rank both matter.
The archived responses include the search, all five candidates, and Lisbon's
value entity. The retained HTTP metadata omits session cookies.
[Captured requests and digests](evidence/facts/) ·
[Wikidata statement ranking](https://www.wikidata.org/wiki/Help:Ranking)

The neighboring authorship replay exposed an inflected Russian book title.
Wikidata search returned no hits for the accusative form. The nominative
form returned several items with the same title, and the book was the only
one of those items with an author claim. Both solvers try the original form
first and can search a feminine head noun in its nominative form as a
fallback. The selected publisher item still has to carry the requested
claim.
[Russian publisher captures](evidence/facts/) ·
[Browser replay](evidence/verification/publisher-fact-live-parity.log)

The pipeline at `727ecf47f` stopped during native compilation because the
scoped how-to resolver still used a cache directory local to its caller.
The resolver now receives that directory explicitly. The Layered workflow on
the same commit reported a stale calculation signature list and two outdated
assertions for the procedure census. The new return path has a recorded
browser probe and the inventory now includes it.
[Pipeline log](evidence/workflows/pipeline-44962926030.log.gz) ·
[Layered CI log](evidence/workflows/layered-44962925794.log.gz)

The next [native check](evidence/workflows/native-job-137041134994.log.gz)
compiled successfully. Its HTTP replay grouped five recurrence requests into
one 60-second deadline and timed out without identifying the active request
(lines 1836–1847). The publisher recurrence path was also fetching raw code
implementations and unrelated concept sources that it later discarded. That
path now checks matched recurrence definitions and their examples directly.
The HTTP replay gives each prompt a bounded request and records the individual
latencies so a later failure identifies the affected input.

The GitHub Docs replay returned steps from the matching capture and preserved
the rejected page in the source trace as `no_steps` (lines 1863–1896). The
replay now verifies the capture digest in the instructions and both outcomes
in the trace. The Chinese currency replay returned the right amount but lost
the publisher URL before `?date=` (lines 1934–1948). A regression exercises
the CJK source label directly in the question filter that removed the URL.
The [Layered CI log](evidence/workflows/layered-44967918592.log.gz) reports
three assertions tied to stale governed source identities. The inventory was
regenerated after rebuilding the callable registry from the merged native
sources. The case comparison records the relocated capital test and the added
currency case. Every previously supported test remains supported.
[Pipeline log](evidence/workflows/pipeline-44967918716.log.gz) ·
[Witness comparison](evidence/verification/governed-baseline-round9-delta.log)

The regional Google Trends probes fetched separate US and DE RSS responses.
The agentic replay checks the response digest, an empty offline cache, and
reuse of the original artifact across tool turns. The coverage report keeps
unresolved prompts available for triage and requires human review of rule
proposals. Historical reports and feeds remain available to explicit replays.
[Publisher evidence](evidence/trends/) ·
[Recipe replay](evidence/verification/google-trends-agentic-recipes.log) ·
[Dreaming replay](evidence/verification/google-trends-dreaming-runtime.log) ·
[Learning cycle replay](evidence/verification/google-trends-learning-cycle.log)

The full JavaScript gate run passed or skipped 134 of 138 checks. The four
failures identified missing seed metadata, translations and assembled
requirements. All four passed when rerun after the corrections.
[Full gate run](evidence/verification/round9-full-gates.log) ·
[Focused rechecks](evidence/verification/round9-gate-rechecks.log)

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
