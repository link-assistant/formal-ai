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

- Bind associative report recipes to an input event graph. The cold replay
  initially produced three observed expressions from compiled historical data.
  Both planners now require a fenced Links Notation block before writing the
  report. Replaying a modified graph yields a different score and the
  lightweight mirror contains none of the eleven historical report bodies.
  [Cold baseline](evidence/verification/baseline-learning-report.log)

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

The same cold replay demonstrated that light speed and painting attribution
still came from bundled prose. These records now use the exact publisher
subject and claim. A dimensional Wikidata quantity resolves its unit from
the unit item and retains its capture digest. Russian painting attribution
tries the painting relation if the exact subject has no book authorship
claim. The source captures stay in the test transport and the case archive.
[Before](evidence/verification/round10-fact-offline-baseline.log) ·
[Publisher replay](evidence/verification/round10-final-live-fact-parity.log)

The Rust creator row also answered without a source and forced the same
person and affiliation into every language. The publisher's programming
language item links its inventor and the inventor item supplies the
language-independent label. Both names must come from their matched items.
A follow-up first resolves its pronoun from conversation history, then
requests the property for the resolved subject. Browser replays exercise
four languages, the follow-up, and an empty offline cache. The browser
explicitly requests the publisher's multilingual default label alongside
the preferred and fallback languages. The live API returns the expected
name under `mul` alone.
[Language-filtered replay](evidence/verification/round10-publisher-label-parity.log) ·
[Default label documentation](https://www.wikidata.org/wiki/Help:Default_values_for_labels_and_aliases/en) ·
[Publisher captures](evidence/facts/) ·
[Live fact replay](evidence/verification/round10-creator-issue-1172-live-fact-parity.log) ·
[Cold cache replay](evidence/verification/round10-creator-issue-1172-seeded-fact-retirement.log)

At head `f6b753d77`, the hosted JavaScript shards found three stale
witnesses exposed by the preceding changes. The profile expected the old
hash of the unchanged rung order, the module map omitted the new native
Trends module, and the return census treated publisher discovery as a new
exit. The local reproductions now check all three. Native server builds
and earlier cases passed. The completion replay reached its request limit
and the server's 90-second lifetime expired during the remaining cases.
The first repair targeted multiple-choice decomposition because a
conjunction can appear inside an option. The meta procedure now preserves
such options with their supplied evidence. The next native replay isolated
the remaining timeouts in the supplied-text summary fixture.
[Layered CI](evidence/workflows/layered-round9-44971850354.log.gz) ·
[Native job](evidence/workflows/native-round9-137053287872.log.gz) ·
[Profile](evidence/verification/round10-hosted-replay-fast-pull-request-production-profile.log) ·
[Module map](evidence/verification/round10-hosted-replay-module-map-twin.log) ·
[Return census](evidence/verification/round10-hosted-replay-issue-1550-procedure-census.log) ·
[Prompt-data replay](evidence/verification/round10-final-issue-1593-prompt-data.log) ·
[Native event parity replay](evidence/verification/round10-final-worker-native-handler-parity.log)

The subsequent full local run passed or skipped 136 of 138 gates. The two
remaining gates passed after updating the generated trace locations and
the status document's core-boundary ledger count.
[Full gate run](evidence/verification/round10-full-gates-final.log) ·
[Trace locations](evidence/verification/round10-debug-stage-source-recheck.log) ·
[Status rendering](evidence/verification/round10-status-render-recheck.log)

### Opening and inception dates

A fresh offline worker described March 31, 1889 as the Eiffel Tower's
public opening. The operator records March 31 as the inauguration and
May 15 as the opening to the public. Wikidata supplies both dates with the
same rank and without qualifiers identifying the events. The opening-date
resolver therefore returns both dates from the publisher record. Its
inception relation uses year precision, which also answers a held-out
question about UNESCO's founding year from that organization's record.
Hindi search first uses the user's spelling and retries with a normalized
borrowed vowel and nukta when required. The item must still match the
normalized subject and carry the requested property.
[Offline baseline](evidence/verification/round11-temporal-baseline.log) ·
[Tower operator chronology](https://www.toureiffel.paris/en/news/history-and-culture/133-years-and-1083-feet) ·
[UNESCO Constitution](https://www.unesco.org/en/legal-affairs/constitution) ·
[Publisher captures](evidence/facts/) ·
[Browser regression](evidence/verification/round11-recheck-issue-1172-live-fact-parity.log)

At head `ca2551662`, native compilation found two references to the fact
parser's old location, and Layered CI found eight TypeScript files out of
sync with their JavaScript sources. Native routing and explanation now
call the relocated helpers. Regenerating the TypeScript projection brought
all 742 files into sync.
[Pipeline log](evidence/workflows/pipeline-round10-44975152457.log.gz) ·
[Layered CI log](evidence/workflows/layered-round10-44975152230.log.gz) ·
[Projection check](evidence/verification/round11-typescript-regen.log)

The full local gate run passed or skipped 134 of 138 checks. The other
four passed after grounding the opening-date template, using literal
recognition strings for the new surfaces, and asserting the complete
answers in the native regression.
[Full gate run](evidence/verification/round11-full-gates.log) ·
[Focused rechecks](evidence/verification/round11-final-gate-rechecks.log)

### Project origin and supplied-text summaries

A cold worker answered questions about who created Formal AI from text in
the installed seed. Both runtimes now use the registered GitHub declaration
for this question. They check the comment and issue identities, the named
project, and whether the logged-in author made the statement about their own
work. The live public response allows cross-origin retrieval and reports the
author as a contributor. The authenticated view reports membership. A
browser replay uses the public response's association, checks the citation
and digest in four languages, then exercises a second project and rejects
unassociated or mismatched declarations.
[Cold baseline](evidence/verification/round12-origin-baseline.log) ·
[Browser replay](evidence/verification/round12-issue-1172-live-fact-parity.log) ·
[Public response](evidence/facts/formal-ai-origin-comment-live.json) ·
[Publisher statement](https://github.com/link-assistant/formal-ai/issues/157#issuecomment-4526577668)

At head `1b195df2e`, native compilation passed. Three native completion
requests hit the ten-second request limit (lines 9414, 9682, and 9703 of the
pipeline log). The fixture ID is `decision` and the instruction is to
summarize a supplied paragraph. The request decomposition could split prose
on a conjunction and recursively solve its fragments before the summarizer
ran. Both runtimes now recognize the summary command at the head and keep
the supplied paragraph together. The replay asserts the decision and both
reasons in English and Russian, in default and agent modes.
[Pipeline log](evidence/workflows/pipeline-round11-44976920425.log.gz) ·
[Local replay](evidence/verification/round12-issue-1593-prompt-data.log)

The same head's Layered CI caught a stale governed native test-source
digest. Regenerating the witness incorporates the current source identities
and the added native date regression, with all 222 supported cases preserved.
The fixture integrity and Unicode source projection replays passed locally.
[Layered CI log](evidence/workflows/layered-round11-44976920393.log.gz) ·
[Witness comparison](evidence/verification/round12-governed-delta.log) ·
[Fixture integrity](evidence/verification/round12-rust-specification-fixture-integrity.log)

All 138 local JavaScript gates passed or reported an expected skip.
[Full gate run](evidence/verification/round12-final-full-gates.log)

### Checked-baggage policy

The cold-cache replay returned a complete stroller policy, including cabin
and fee conditions, without retrieving the airline's policy. Those conditions
came from four installed translations of the same written answer. The airline
currently lists a stroller, carrycot, or infant car seat under domestic
checked baggage. Its travelling-with-infants page separately discusses
equipment on board. Quoting the matched checked-baggage passage avoids
extending the passage to cabin, fee, and itinerary rules it does not state.
Direct requests from this workspace failed to retrieve the page. The public
publisher page was inspected through the search tool, and deterministic
replays use a reduced HTML excerpt in the tests.
[Cold baseline](evidence/verification/round13-policy-baseline.log) ·
[Air India checked baggage](https://www.airindia.com/in/en/travel-information/baggage-guidelines/checked-baggage-allowance.html) ·
[Air India guidance for travelling with infants](https://www.airindia.com/content/air-india/in/en/travel-information/first-time-flyers/expected-mothers-infants.html)

The installed record carries a source reference, a heading, and evidence
terms. Both solvers reuse the practical-help section reader. The browser
replay covers changed publisher text, an unrelated section, four prompt
languages, cold and warm offline requests, and an independently registered
carrier. The native regression checks the same capture and its recorded hash.
[Browser replay](evidence/verification/round13-final-issue-1172-live-fact-parity.log) ·
[Native regression source](../../../rust/tests/unit/specification/stroller_baggage_fact.rs)

### Native publisher recurrence routing

At head `d7ecf560a`, the native completion replay passed 84 cases and
skipped one expected case. Its following recurrence replay recorded two
English factorial requests reaching the 30-second test limit (pipeline
lines 9795 and 9806). The Russian and Hindi factorial requests returned
sourced functions. The held-out Fibonacci request returned a catalog
example that invoked a fixed demo argument and omitted the publisher
citation (lines 9833–9915). Layered CI passed on this head.
[Native pipeline](evidence/workflows/pipeline-round12-44979226068.log.gz) ·
[Layered CI](evidence/workflows/layered-round12-44979225868.log.gz)

The original publisher branch ran after the native solver had selected a
matching program catalog task. It could miss a callable outside that
catalog or allow a general program handler to respond first. The native
solver now attempts a verified publisher recurrence once it has bound
the request to a callable Python artifact. The phrase extractor removes
grammatical scaffolding before it limits candidate spans, preserving
subjects such as “nth Fibonacci number.” The general synthesis path
remains available for requests the publisher cannot support.

The English timeouts did not identify the stage where the request
stalled. The recurrence replay now enables trace events for publisher
and general synthesis and reports the final observed stages, plus the
individual request time and the source failure reason when available.
The installed 0.355.27 reference CLI returned a cold factorial fallback
in under three seconds with the same empty-cache policy. That run
establishes a reference timing for a different build. The native
regression on the new commit is verified by its own hosted pipeline.
[Reference CLI run](evidence/verification/round13-reference-cli-cold-recurrence.log) ·
[Native regression](../../../rust/tests/web/issue-1396-native.test.mjs)

The full local JavaScript gate run passed or skipped 135 of 138 checks.
The other three passed after spelling out the recurrence helper's name
and documenting the complete policy response for each tested language.
The regenerated native trace locations and test-source witnesses also
passed their focused checks.
[Full gate run](evidence/verification/round13-final-gates.log) ·
[Focused rechecks](evidence/verification/round13-gate-rechecks.log) ·
[Fixture integrity](evidence/verification/round13-corrected-rust-specification-fixture-integrity.log) ·
[Unicode source projection](evidence/verification/round13-corrected-rust-specification-unicode-identifiers.log)

## Film release timelines and the offline synthesis delay

The baseline browser answered the original Spider-Man request with twelve
records from the installed timeline and zero publisher requests. It returned
unknown for a Harry Potter request with the same structure. The installed
registry now supplies query scope and wording. The Wikidata Query Service
supplies membership and dates at request time through the shared capture
client. The former snapshot remains a test input, including its original
release classification and freshness date. [Cold baseline](evidence/timelines/cold-baseline.log) ·
[Historical replay](../../../rust/tests/fixtures/release-timeline/spider-man-replay.lino)

The recorded live query returned twelve rows in English and Russian for
Spider-Man and eight for Harry Potter. `P179` and `P527` cover series and
subseries membership. The film, distributor, and media filters preserve the
title-role benchmark scope and exclude shorts, television, and extended
editions. The answer uses the earliest date asserted by the publisher, which
can differ from a single market's release date. A unique item label grounds
an unregistered series. Ambiguous item matches, excessive rows, and rows
about the wrong subject produce an explicit evidence gap. The main WDQS
endpoint responded in the test environment after the legacy endpoint timed out
and the alternate endpoint returned HTTP 403.
[WDQS API](https://www.wikidata.org/wiki/Wikidata:SPARQL_query_service/API) ·
[Publisher query and response metadata](evidence/timelines/) ·
[Browser replay](evidence/verification/round14-issue-0918-handler-rules-batch.log)

The next native run verified all four supplied recurrence cases. Its only
failure was the absent-capture test. The trace shows the publisher attempt
returning in 202 ms and general synthesis starting 285 ms later, with no
completion event before the 30-second test deadline. The call path could start
enumerating typed Python candidates before checking for examples it could use
to test them. The runtime now starts that enumeration only when the caller,
recognized requirements, or a verified source provide an executable oracle.
Further opt-in events identify catalog acquisition, concept discovery,
initial composition, and sequence-source expansion if the delay recurs.
[Native run](https://github.com/link-assistant/formal-ai/actions/runs/44981802284) ·
[Trace with log line numbers](evidence/verification/round13-native-synthesis-phases.log) ·
[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/44981801921)

The next native run passed all recurrence cases, including the empty-cache
request. The registered timeline passed, and the unregistered series returned
unknown even after both captures were present. `fact_subject` admitted the
registered record and omitted the publisher series operand, so the
contextual handler never received the open-series request. Claim admission
and retrieval now use the same frame parser. The HTTP replay includes normal
sentence punctuation and tests the unregistered series before and after
inserting the verified captures. [Native failure and passing recurrence cases](evidence/workflows/pipeline-round15-44985653709.log.gz) ·
[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/44985653306)

## Regional feeds in the browser worker

The browser's learning directive returned a fixed acknowledgment and issued
zero source requests. Native and Node recipes already read a regional RSS
feed. The worker now uses the same parser as Node, applies source settings,
and replays generated prompts through local reasoning. The response reports
the captured topics, source digest, and count of prompts the local rules
solved. It leaves rule adoption to the existing review workflow.
[Browser baseline](evidence/verification/round16-baseline-browser-trends.log) ·
[Browser replay](evidence/verification/round16-issue-1173-route2-probes.log)

The German feed returned RSS to a request with the hosted browser's Origin
header. The response lacked `Access-Control-Allow-Origin`. The worker can
fall back to its configured capture component when direct browser access
fails. The replay supplies this path for Germany and direct retrieval for
the United States. It checks verified offline reuse, a fresh offline miss,
a disabled source, and rejection of a non-RSS response. No feed is installed
with the worker. [Selected publisher headers](evidence/trends/publisher-origin.headers.gz) ·
[Publisher response](evidence/trends/publisher-origin.xml) ·
[Node recipe replay](evidence/verification/round16-agentic-recipes-b.log)

An adjacent routing replay captured a Python declaration with no prose
language cue. The worker's function-specification claim now accepts a
syntactic function declaration as evidence, as the native reader already
does. [Before](evidence/verification/round16-function-signature-baseline.log.gz) ·
[After](evidence/verification/round16-issue-1175-claim-routing.log)

The four focused JavaScript suites passed 58 tests. The full gate run passed
or skipped 137 of 138 checks. The seed audit found two missing metadata
fields on the new RSS concept. The concept now supplies its unit and an
example, and the audit and adjacent checks all passed on recheck.
[Full run](evidence/verification/round16-local-gates.log) ·
[Rechecks](evidence/verification/round16-gate-rechecks.log)

## Callable sorting and historical learning reports

Sorting directions originally selected complete response snippets in the
handler table. A function request now binds the desired order and target
language to list transformations. The test scaffold receives the caller's
sample values or generates a sequence that an identity function could not
sort correctly. The browser suite runs the rendered functions and tests,
including repeated and fractional inputs and literals with signs and leading
zeroes. The native rule and conversational handler share the list interpreter.
[Browser regression](evidence/verification/round17-sorting-code-plans.log) ·
[Adjacent rule tests](evidence/verification/round17-handler-rules.log)

The native learning cycle used eight historically captured learning reports
on a default invocation. The CLI now asks for a report path and accepts
multiple files in the order supplied. It rejects a file that declares a
different frontier. Historical replay commands in the benchmark guide name
their inputs explicitly. The previous compile inputs are recorded in the
[baseline inventory](evidence/verification/round17-frontier-package-baseline.log).
They are absent from the new
[Cargo source package inventory](evidence/verification/round17-frontier-package-inventory.log.gz).

The hosted native run passed the film timeline replay with both required
captures and an unregistered series. Its JavaScript jobs reported a stale
role registry and an absent translation projection for the shared RSS
parser. Both generated artifacts have been refreshed. The two translation
suites passed 29 tests locally.
[Pipeline](evidence/workflows/pipeline-round16-44987974668.log.gz) ·
[Layered CI](evidence/workflows/layered-round16-44987974451.log.gz) ·
[Translation projection](evidence/verification/round17-translate-trends.log)

## Programming language contracts and publisher documentation

A request for a small output program used compiled templates, an example
oracle, and publisher responses preloaded into the runtime. Removing an
explicit template could leave the other two mechanisms answering from the
same example. A fresh offline request now composes an output operation
from the requested language's syntax and the caller's quoted text. The
canonical greeting remains the operand of the named Hello World exercise.
An online request can acquire the publisher example, bind its output and
command contract, and cite its source and content digest. The raw capture
cache may reuse the document without creating a reviewed execution recipe.
The Node server renders the execution status for this program separately
from the historical toolchain observation.

The original publisher captures, with their digests, are now under
`rust/tests/fixtures/coding-discovery`. Test transports explicitly supply
them to the resolver. The fresh worker catalog test covers language and task
parity and the missing capture case. The old dedicated oracle returns no
installed answers. Its capture reconstruction and public-source
registrations can still work with explicit documents.
[Worker catalog regression](evidence/verification/round18-worker-catalog.log) ·
[Publisher and procedure cache replays](evidence/verification/round18-focused-replay.log)

The governed specification replay retained all 222 supported case identities.
Its inventory still contains 963 unsupported cases, with four renamed to
describe the execution-status assertions they now make. The generated native
projections retain the same delivery boundary. The local gate replays also
verify that the source status page and hardcoded-language ceiling reflect the
removed literals. The full run completed 135 of 138 gates locally. All three
reported failures passed in the focused rerun after the two generated surfaces
were corrected.
[Full local run](evidence/verification/round18-local-gates.log)

The prior hosted run failed on private numeric-value visibility, generated
requirement status, the notation ledger title, and stale native source
fingerprints. The archived job logs locate the failures at lines 1257–1291,
765–768, 4164 and 4632, and 2716–2759 respectively.
[Type check](evidence/workflows/job-round17-137109013250.log.gz) ·
[Requirement status](evidence/workflows/job-round17-137108905081.log.gz) ·
[Notation](evidence/workflows/job-round17-137108948497.log.gz) ·
[Source fingerprints](evidence/workflows/job-round17-137108948526.log.gz) ·
[Pipeline](evidence/workflows/pipeline-round17-44990051689.log.gz) ·
[Layered CI](evidence/workflows/layered-round17-44990051323.log.gz)

### Script requests and execution inputs

A language-only script request previously acquired the Hello World exercise
as its missing task. The file-listing and stdin-copy answers declared stdout
chosen by the catalog. The Node server could then replace the worker's report
with an installed toolchain observation that applied to a different program.
The baseline captures these paths on a fresh worker. An explicit print
request now binds the supplied text and the language named in the instruction.
A request that names only a language asks what the program should do.
[Cold baseline](evidence/verification/round22-program-baseline.log) ·
[Program replay](evidence/verification/round22-program-replay.log)

Listing takes its path from the working directory or the caller's argument.
Copy takes its bytes from standard input. The expected-output block is absent
until there is a concrete input and output to check. The browser can report a
sandbox run when it completed. Native and Node answers identify unchecked code
as unprobed. The regression creates a directory with known entries and invokes
the generated program on that directory. Another regression pipes a byte stream
including a null byte and checks it without decoding or normalization.
[Behavior replay](evidence/verification/round22-issue-1021-behaviour-range.log) ·
[Script operands](evidence/verification/round22-script-print-operand.log) ·
[Server execution report](evidence/verification/round22-server-evidence-links.log)

The reviewed procedure reader also used the repository fixture path as its
fallback. Both native and Node now resolve it under the selected application
cache root. An explicit `FORMAL_AI_PROCEDURE_CACHE` still selects a file. The
browser starts without a reviewed file. A cache row describing one example
cannot replace a file or stream operation whose exact input is still unknown.
[Procedure cache replay](evidence/verification/round22-issue-1165-server-procedure-cache.log)

The Layered CI replay on the previous head compared the new TRIZ summary to an
old generated architecture map. The generator refreshed the map and the twin
compares its output byte for byte with the committed document.
[Hosted log, lines 4486–4528](evidence/workflows/job-round21-layered-137140947544.log.gz) ·
[Local replay](evidence/verification/round22-module-map-twin.log)

## Computer-use plans and observed HTTP responses

The prior planner selected a recorded operation sequence from the prompt.
The active-customers request immediately scheduled a write containing three
invented customer records, although the caller supplied no data. The HTTP
adapter likewise mapped four fixture URLs to installed files and simulated a
submission response from a fixed token. The original recordings are valid
replay inputs and now live with the tests.
[Cold baseline](evidence/verification/round20-computer-use-baseline.log)

Both tool planners accept a bounded list of caller-specified operations and
validate every entry before scheduling the first effect. The CLI can also
learn operations from a caller-supplied Links Notation document and emit a
plan for another client. HTTP operations require an operator-configured
origin and the existing network permission. A POST also requires effect
confirmation. A successful response records the status, media type, digest,
and cache path using the bytes returned by the named server. The replay
exercises the original ten requests, 48 held-out localized requests, and
live loopback GET and POST responses, including origin and status failures.
[JavaScript replay](evidence/verification/round20-agentic-computer-use.log)

The Cargo source manifest retains the policy seed and excludes the archived
task corpus and response files. The pull request's native job also runs the
four existing #707 test executables with the loopback publisher.
[Package inventory](evidence/verification/round20-cargo-package-inventory.log.gz)

On commit `66169b8fa`, native checks reached the HTTP receipt test and it
passed. Two later assertions failed because the native workflow's sparse
checkout omitted the harness and the localization test's mostly English
prompt was detected as English. The Layered CI policy test also found
the new native command missing from its inventory, and its subprocess
test flagged the HTTP adapter's direct call to Node's child-process API.
The workflow now includes the harness script and its policy inventory
matches the commands it runs. The Hindi test uses a Devanagari prompt, and
the HTTP adapter uses the shared subprocess supervisor for request input
and both output streams.
[Native job, lines 1703–1734](evidence/workflows/job-round20-native-137137583566.log.gz) ·
[Layered policy job, lines 2113–2149](evidence/workflows/job-round20-layered-137137539167.log.gz) ·
[Layered subprocess job, lines 2719–2739](evidence/workflows/job-round20-layered-137137539177.log.gz)

Layered CI on `3362640bc` found outdated TypeScript translations for the
associative report and planner edits. The job log lists all fourteen files at
line 302. The regenerated TypeScript projection passes the parity check.
[TypeScript job](evidence/workflows/job-round19-137131958325.log.gz) ·
[Local parity check](evidence/verification/round20-typescript-parity.log)

The full local gate run had five failures. The rechecks cover all five after
spelling out new bindings, adding the Spanish GET description, updating the
mirrored seed and runtime inventory, and recording the smaller closure gap.
[Full gate run](evidence/verification/round20-full-gates.log) ·
[First rechecks](evidence/verification/round20-gate-rechecks.log) ·
[Final rechecks](evidence/verification/round20-final-gate-rechecks.log) ·
[JavaScript-to-Rust generation](evidence/verification/round20-js-rust-generation.log)

## TRIZ worksheets from stated demands

The old runtime stored the twenty cases for issue #901 beside the resolution
methods. Its match score counted overlapping words of at least five letters.
For an unfamiliar contradiction with no overlapping words, both solvers used
the first three rows. Asking about latency versus acoustic damping returned
pill dispensing, wheat grains, and lawn care. An incomplete paradox request
repeated the same cases while the claim router recorded a refusal.
[Before-fix replay](evidence/verification/triz-baseline.log) ·
[Original benchmark request](https://github.com/link-assistant/formal-ai/issues/901)

Both solvers now extract the two stated demands and ask the caller for missing
information before printing a worksheet. The worksheet uses the original terms
and the installed resolution methods. It asks when and where the requirements
apply and how to measure success. When the caller provides priorities, the
existing selection heuristics can derive a weight from the requirement clauses.
MATRIZ technical materials start from competing demands, their operating time
and zone, and hypotheses about how to separate them. The Altshuller Foundation
publishes the inventive principles as reusable methods. A design still needs
tests or observations from its own domain to establish that it satisfies both
demands.
[MATRIZ study materials](https://wiki.trizstudies.org/Problem-solving-tools.html) ·
[Altshuller Foundation principles](https://altshuller.ru/world/eng/technique1.asp) ·
[Altshuller Institute introduction](https://aitriz.org/pages/triz)

The regression parses all twenty historical contradictions loaded from a test
file. It also exercises held-out pairs in five languages, preserves literal
braces and ratio punctuation, and checks that incomplete or repeated demands
produce a clarification. The native PR workflow repeats held-out and
missing-input cases through the public gateway.
[Historical cases](../../../rust/tests/fixtures/triz/benchmark-tasks.lino) ·
[Browser and gateway replay](../../../rust/tests/web/issue-901-triz-parity.test.mjs) ·
[Local test log](evidence/verification/triz-tests.log) ·
[Claim routing test](evidence/verification/round21-claim-tests.log)

The full local run identified two file-size checks on an intermediate seed
layout. Moving the missing-demands intent to the engine report registry kept
both registries below the line limit. The targeted rerun passed both checks.
[Full gate log](evidence/verification/round21-local-gates.log) ·
[Targeted rechecks](evidence/verification/round21-local-gate-rechecks.log)

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

## Wikisource and Wikidata evidence in agentic recipes

The historical formalization trace showed the same seven-sentence synopsis on
successful fetch and source failure. It selected the nine-primitive result
before verifying any publisher response. The meaning-detail recipe had the
same failure mode for tomato and potato, with three embedded lexeme responses
standing in for a complete request. An unknown word could even borrow the
tomato recipe before the earlier routing correction.

On November 22, the direct MediaWiki parse endpoint returned the public-domain
Pushkin poem at 03:54 UTC. Its first poem block starts with the primary text.
A later poem block in the same page is a variant in the commentary. An
experiment with raw wikitext found the main verse under a template and a
literal poem tag in the variant. The rendered article gives the extractor an
explicit class and ordered body to verify. The historical synopsis remains
an explicit test input for the nine-primitive coverage requirement.
[Wikisource HTTP evidence](evidence/lexemes/fisherman.headers) ·
[Historical synopsis fixture](../../../rust/tests/fixtures/publisher-agentic/fisherman-synopsis.txt) ·
[Publisher response fixture](../../../rust/tests/fixtures/publisher-agentic/fisherman.json) ·
[MediaWiki parse API](https://www.mediawiki.org/wiki/API:Parsing_wikitext)

Live Wikidata requests at 03:18, 03:54, and 04:13 UTC returned the requested
tomato, potato, and apple lexemes. The first senses of L7993, L3526, and L3784 refer
to edible food items. Their second senses contain the plant item required by
the tomato and potato declarations. L170542 supplies declared Russian word forms
and carries no matching plant sense. The required sense is checked by item
claim, including when its number changes. One batch supplies all declared
forms, and a missing or mismatched response leaves a visible source gap.
[Acquisition endpoints and checksums](evidence/lexemes/README.md) ·
[Wikibase lexeme data model](https://www.mediawiki.org/wiki/Extension:WikibaseLexeme/Data_Model)

The existing source client provides bounded acquisition and cache identity.
The tool-history parser associates responses with their request URL. Direct
Lexeme API queries use stable registered IDs and avoid committing copies.
An open-ended request can still use the general research lane when no
registered meaning is identified. The publisher replay verifies the poem,
detects the commentary variant, and checks the same grammatical form logic
for three concepts. It also supplies absent, partial, and wrong responses
and verifies that the planner ends before creating a document.
[Recipe verification](evidence/verification/round15-agentic-recipes-a.log) ·
[Agentic server verification](evidence/verification/round15-agentic-server.log) ·
[Cargo source archive inventory](evidence/verification/round15-cargo-package-inventory.log)

The first hosted run on the release-timeline change stopped in two different
places. The native type checker reported two calls passing an owned string
where a borrowed fare qualifier was required. Both are now borrowed. The two
browser replay shards stopped before dispatch because the worker boot manifest
still held a previous digest for the changed timeline script. The maintained
generator refreshed the worker and server manifests before the next replay.
[Native log, lines 8740–8778](evidence/workflows/pipeline-round14-44983621991.log.gz) ·
[Browser log, lines 8589 and 9779](evidence/workflows/layered-round14-44983621838.log.gz) ·
[Worker prerequisite verification](evidence/verification/round15-worker-script-prerequisites.log)
