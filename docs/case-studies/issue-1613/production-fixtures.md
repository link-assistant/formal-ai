# Caller inputs and explicit test data

[Case study](README.md) · [Repository audit](repository-audit.md) ·
[Issue #1613](https://github.com/link-assistant/formal-ai/issues/1613) ·
[PR #1614](https://github.com/link-assistant/formal-ai/pull/1614)

## Composite programs

The same program bodies were installed in the native catalog and browser.
Their selection recognized a class of request, but their output came from
example data inside the programs. The budget report projected one income and
compared fixed city costs. The travel report ranked four example cities with
pre-filled visa assertions and fares. The portfolio reported built-in prices
for four holdings. Source metrics compared the generated program with a
prepared paragraph and always awarded the same result.

Each program now has an input contract. Budget and travel read JSON scenarios
with assumptions, observations, and references. The portfolio reads holdings
and price observations from files or a caller-specified HTTPS endpoint. Source
metrics reads the supplied reasoning file and compares measured counts. The
browser test executes the generated programs using Python and Rust. It changes
the contribution and city, trip budget and destination, holdings and prices,
and reasoning file, then checks each resulting report. The first run with
these assertions failed on the old templates. The follow-up passed all four,
and the full catalog replay passed 14 tests.
[Before](evidence/verification/round25-before-blueprint-final.log.gz) ·
[After](evidence/verification/round25-after-blueprint-routing.log) ·
[Catalog replay](evidence/verification/round26-final-issue-0921-worker-coding-catalog.log)

The programs consume observations from the request's environment. This
keeps market and visa data refreshable and gives cold callers the fields and
run command they need. Binding a single provider into the template would add
a service and schema dependency for every generated program. A caller can
select the source and replay the same program against a new scenario.

## Creative and knowledge requests

The creative seed installed 269 rows of Rome places and transfers, sleep
advice, and rhyme lines. Their reader recognized the topic and immediately
returned selected rows. The historical replay with evidence supplied to
the old advice handler exposed that it still returned the installed guidance.
The scheduler now places the supplied visits within cited opening windows and
transfer constraints. The advice compositor orders the supplied statements
by their declared evidence grades. The verse compositor checks the requested
number of lines and verifies their ending classes against the supplied
pronunciation observations. Its regressions include a six-line scheme with
three distinct rhyme classes. The original examples are explicit fixture
inputs. Evidence URLs are checked for HTTPS, host, and absence of user
credentials. The source assertions and grades come from the caller.
[Before](evidence/verification/round26-before-creative.log.gz) ·
[Creative replay](evidence/verification/round26-final-issue-1178-creative-parity.log) ·
[Observation fixtures](../../../rust/tests/fixtures/creative-observations/)

An itinerary can use hours common to its requested days. Trips with different
opening days need a schedule with dated observations. MediaWiki Geosearch
could supply nearby candidates but does not supply opening windows or
transport durations. OpenStreetMap stores opening hours and optional source
attribution, which can support collection when that source is available.
An evidence source like the CDC gives a publisher location for sleep
guidance. Matching and grading the actual guidance still requires the
recommendation evidence. The general procedural source walk can retrieve
a published guide separately.
[MediaWiki Geosearch API](https://www.mediawiki.org/wiki/API%3AGeosearch) ·
[OpenStreetMap hours](https://wiki.openstreetmap.org/wiki/Key%3Aopening_hours) ·
[OpenStreetMap hours provenance](https://wiki.openstreetmap.org/wiki/Key%3Asource%3Aopening_hours) ·
[CDC sleep guidance](https://www.cdc.gov/sleep/about/)

The summary path used to admit just three registered subjects. It now extracts
the requested subject, looks it up through the shared project and concept
registries, and can first try publisher discovery for article requests.
WebAssembly and Wiktionary verify lookup outside the old list. The project
path reads repository metadata. The concept fallback resolves the registered
definition for the extracted term. Brainstorm categories
now choose composition operations, which combine the supplied topic with
generic prompts for designs. Naming requests use the existing morphological
composer. Missing topics elicit a clarification. The browser tests cover
distinct ordinary topics as well as the previous promotion routes.
[Summary replay](evidence/verification/round26-summary-after.log) ·
[Handler replay](evidence/verification/round26-final-issue-0918-handler-rules-batch.log) ·
[Claim routing replay](evidence/verification/round26-final-issue-1175-claim-routing.log)

### Roleplay and conversation references

`Roleplay as a field botanist: Mosses absorb water across their surfaces.`
previously returned a stored explanation of relativity for a default persona.
Both runtimes now read a role and facts from the message. An exact identity
alias can still supply a canonical name and Wikidata identifier. The answer
labels the facts as caller supplied. If either field is absent, the handler
asks for the missing input. The replay covers an unregistered role, a
recognized alias, placeholder characters in the input, and a full-width
colon.
[Before](evidence/verification/round27-roleplay-baseline.log.gz) ·
[After](evidence/verification/round27-roleplay-final.log)

The previously stored Rust comparison also appeared for `Is it installed
here?` after a turn about Rust. Fact lookup still receives the rewritten
question when supported. Otherwise the engine reports which conversation
reference it resolved and that it needs evidence to answer the follow-up.
The regression checks the unrelated question and preserves the test for
resolution across an intervening topic.
[Before](evidence/verification/round27-coreference-baseline.log.gz) ·
[After](evidence/verification/round27-coreference-final.log)

## Repository URLs and descriptions

The first HTTP regression requested the issue path of one registered repository
and the README path of another. Both answers used the repository's installed
description, and the transport received only the third, unregistered URL.
Explicit fetch now acquires the requested URL under the same capture policy
for all three and records the observed HTTP result. Running without a
transport cannot substitute the repository description.
[Before](evidence/verification/round27-project-fetch-baseline.log.gz) ·
[After](evidence/verification/round27-project-summary-parity-final.log)

A second regression found that an offline request to summarize Hive Mind
returned a bundled statement with the repository credited as its source.
The same statements supplied answers for promoted project lookups. The
registry now holds repository identity and lookup metadata. Both runtimes
verify `full_name` and `html_url` in the GitHub repository response before
using a nonempty publisher description. They record the capture's URL,
timestamp, digest, and cache status. An offline capture miss or a conflicting
repository identity produces a localized gap. The browser replay covers a
cold miss, an observed response, offline reuse, changed descriptions, and
an identity mismatch. The old weighted statements are explicit inputs for
the historical Rust source-test target.
[Before](evidence/verification/round27-project-metadata-baseline.log.gz) ·
[After](evidence/verification/round27-project-summary-parity-final.log) ·
[GitHub repository endpoint](https://docs.github.com/en/rest/repos/repos#get-a-repository)

Fetching a README could provide longer evidence when a publisher supplies no
description. It would also need an additional capture, Markdown extraction,
and language attribution. The repository metadata already exposes the short
description and the two identity fields needed for the lookup. The separate
README summarization API still accepts caller-supplied documents.

## Installation and verification boundaries

The dialogue seed and the full industry and coding-modification benchmark
suites were mirrored into the native package. The dialogue tests now read the
training examples by path. The promotion runtime reads a descriptor with only
the suite identity, check command, and pass threshold. The benchmark runners
still use the original test cases. A regression checks the unchanged gates and
the removal of benchmark cases from the runtime mirror. Native Python draft
verification now generates small typed examples and computes their expected
results independently. It requires an explicit output contract or a bounded
derivation for runnable programs. The maximum derived range has 1,000 values.
[Promotion replay](evidence/verification/round26-final-server-learning-trace.log) ·
[Verification implementation](../../../rust/src/coding/composition/verification.rs) ·
[Native verification tests](../../../rust/tests/unit/coding_discovery/no_memorization.rs)

The Cargo package manifest listed 1,290 files. It omitted the old dialogue
seed, both promotion benchmark suites, captured GitHub issues, and the
creative and blueprint replay fixtures. It retained the lightweight promotion
descriptors and the native verification module.
[Cargo package inventory](evidence/verification/round26-cargo-package-inventory.log.gz)

The first full JavaScript preflight caught the updated source and response
inventory limits and the missing exact-answer fixtures. The final preflight
completed all 138 gate entries, with 86 run locally and 52 reserved for CI.
Formatting checks passed for the edited native source and tests.
[First preflight](evidence/verification/round26-local-gates-first-pass.log) ·
[Final preflight](evidence/verification/round26-local-gates-final.log)

## Hosted and development traces

The hosted head `b7dfed1ef` produced matching failures in native and Layered
CI. The localization replay assumed one unknown term and its position in the
list. Russian legitimately left an additional term unresolved. The revised
test checks the required unknown span while allowing other unsupported terms.
All 85 prompt-data cases passed locally. Security CI also reported
RUSTSEC-2026-0592 in `url` 2.5.8. The lockfile now selects the fixed 2.5.9.
[Native log, lines 2025–2045](evidence/workflows/native-round24-137170626097.log.gz) ·
[Layered log, lines 5562–5648](evidence/workflows/layered-round24-45010155702.log.gz) ·
[Security log, lines 533–550](evidence/workflows/security-round24-45010155940.log.gz) ·
[Prompt-data replay](evidence/verification/round26-issue-1593-prompt-data.log) ·
[RustSec advisory](https://github.com/RustSec/advisory-db/blob/main/crates/url/RUSTSEC-2026-0592.md)

The second local authorship attempt used Agent CLI 0.27.0 and the branch's
JavaScript server. The request asked for a named Python program that would
identify test fixture paths and run against two examples. The server accepted
the chat request and chose an unknown plan. It returned a generic JavaScript
preview with no tool calls. The CLI timed out and the output file was absent.
The server's file-descriptor error was recorded during teardown, after the chat
exchange. This attempt contributes zero authored leaves. The earlier workflow
never started the model because its image could not be pulled.
[Retry request](evidence/formal-development/round26-request.txt) ·
[Agent transcript](evidence/formal-development/round26-agent.log.gz) ·
[Server trace](evidence/formal-development/round26-serve-427000.lino.gz) ·
[Initial attempt](README.md#timeline-on-2026-11-21-in-utc)

The next Agent CLI run narrowed the task to extracting the runner and both
thresholds from the original, pre-extraction coding benchmark manifest. The
session read the supplied manifest, wrote the expected-answer fixture, and
read it back with matching bytes. The committed regression independently
compares the three fields with the shipped descriptor and runs the same
Formal AI plan against the archived manifest to reproduce the fixture
byte-for-byte.
[Request](evidence/formal-development/promotion-oracle/request.txt) ·
[Agent transcript](evidence/formal-development/promotion-oracle/agent.log.gz) ·
[Tool and read-back trace](evidence/formal-development/promotion-oracle/server-memory.lino.gz) ·
[Replayed fixture](../../../rust/tests/fixtures/benchmark-promotion/coding-modification-contract.lino) ·
[Regression results](evidence/verification/round26-promotion-oracle-replay.log)

The browser package regression found that an old `js/source-cache` directory
would enter the npm archive. A manual preparatory change named the existing
asset exclusion list without changing its entries. The Agent CLI then used the
branch's JavaScript planner to read the generator, add the capture directory to
the list from a natural-language request, and verify the written file by hash.
The commit uses those exact bytes. An independent replay verifies the same hash,
and the packaging regression checks both the archive and offline manifest.
These sessions verify two authored leaves. The 20% contribution floor for the
full change remains unverified.
[Request](evidence/formal-development/browser-cache-exclusion/request.txt) ·
[Agent transcript](evidence/formal-development/browser-cache-exclusion/agent.log.gz) ·
[Tool and digest trace](evidence/formal-development/browser-cache-exclusion/server-memory.lino.gz) ·
[Regression results](evidence/verification/round27-browser-exclusion.log)

The strict contribution checker credits the browser distribution commit.
The published promotion-fixture commit used a directory of compressed
evidence and omitted the model trailer. A follow-up commit retracts that
metric claim while preserving its recorded session and independent replay.
[Browser model and session](evidence/formal-development/browser-cache-exclusion/session.txt) ·
[Strict contribution check](evidence/verification/round27-contribution-verified.log)

Hosted head `5237db3db` reported three actionable failures. Native type
checking found a caller of the removed `pick_topic` method. Both runtimes now
use the summary subject extractor when checking the claim operand. Layered
CI found seventeen stale TypeScript projections. The maintained translator
regenerated them along with the current worker changes, and its byte check
passed. The Security job found the same URL advisory in the separate
experiment lockfile. The experiment now resolves `url` 2.5.9, and the local
audit completed across both lockfiles.
[Native log, lines 1261–1275](evidence/workflows/native-round27-137191835228.log.gz) ·
[Pipeline log, lines 9659–9673 and 9995–9998](evidence/workflows/pipeline-round27-45015541253.log.gz) ·
[Layered log, lines 9153–9155](evidence/workflows/layered-round27-45015541051.log.gz) ·
[Security log, lines 535–549](evidence/workflows/security-round27-45015541174.log.gz) ·
[TypeScript check](evidence/verification/round27-translate-es-check.log) ·
[Local advisory check](evidence/verification/round27-cargo-audit.log)

The local gates caught a second verification gap. Combining the two native
HTTP cases into a loop made those checks unextractable by the JavaScript
specification runner. The native specification now keeps a simple request case
for each path and an exact-response check for a cold cache. A browser replay
also confirms that both offline and network-denied settings prevent requests
and that an explicit fetch can reuse verified cached bytes. The native and
browser routes use the same localized cache message.
[Specification replay](evidence/verification/round27-specification-worker.log) ·
[Browser request replay](evidence/verification/round27-project-summary-parity-final.log)

The English Moon page had 195 unknown terms on both the base and branch.
The previous fixture expected 196. The branch also preserves the last of the
62 factual statements on the round trip. The statement renders an inclusive
range as `count to three`. The base reparses that phrase as a particular
program task. The branch recognizes the shared range and keeps the number
as a separate term. An isolated replay computes the counts from the archived
page and compares the changed statements. The localized test cases retain
exact reports for all five languages.
[Base test](evidence/verification/round27-page-formalization-main.log.gz) ·
[Statement comparison](evidence/verification/round27-page-vocabulary-delta.json) ·
[Updated test](evidence/verification/round27-page-formalization-final.log) ·
[Replay script](../../../experiments/issue-1613-page-vocabulary/replay.mjs)

Two Agent CLI attempts to remove the HTTP shortcut produced no write. In the
browser module attempt, the read crossed the configured context window and
the replacement prompt after compaction lost its edit request. In a smaller
Rust input, the session reached the same read three times before the retry
ceiling ended the operation. The working tree edits were manual, and the
attempts contribute zero autonomous-authorship credit.
[Browser attempt](evidence/formal-development/browser-fetch-authorship/agent.log.gz) ·
[Rust attempt](evidence/formal-development/native-fetch-module-authorship/agent.log.gz)

A narrower session asked the Agent CLI to add a Spanish roleplay trigger to
the seed file. Formal AI read the eleven-line file and returned its contents
without issuing an edit. The CLI exited successfully and the sandbox diff
was empty. The attempt contributes zero autonomous-authorship credit.
[Request](evidence/formal-development/roleplay-trigger-authorship/request.txt) ·
[Tool trace](evidence/formal-development/roleplay-trigger-authorship/server-memory.lino.gz) ·
[Outcome](evidence/formal-development/roleplay-trigger-authorship/outcome.txt)

The merge from `main` combines the early Docker startup diagnostic with the
isolated, offline launch used by the cold-cache check. The governed witness
contains the same Cargo manifest digest as `main`. Comparing it with the
pre-merge branch shows one more executable specification case, from the
repository lookup regressions, and updated source digests for the lookup and
roleplay specifications. The 34 governed witness tests, seven Docker source
tests, and 23 workflow build-policy tests passed.
[Witness comparison](evidence/verification/round27-governed-merge-delta.json) ·
[Governed tests](evidence/verification/round27-governed-merged-test.log) ·
[Docker tests](evidence/verification/round27-container-merged-test.log) ·
[Build policy tests](evidence/verification/round27-build-policy-merged-test.log)

The subsequent `v0.356.5` release merge updated the Cargo manifest digest in
four test witnesses. Comparing all 223 cases confirmed that this was the only
inventory change, and all 34 governed witness tests passed again.
[Release comparison](evidence/verification/round27-governed-release-merge-delta.json) ·
[Release merge tests](evidence/verification/round27-governed-release-merged-test.log)

## System self-explanation

The agentic recipe attached fresh citation IDs to five fixed paragraphs. One
said that the system's code was present in its Links diagnostics. The cited
module instead describes a navigation representation that cannot reconstruct
the source files. The replay substitutes one owned module in the host and asks
the normal planner for the self-explanation. On the old code the citation ID
changes and the old paragraph persists. The native and JavaScript recipes now
derive each section from the leading module documentation of its primary
source citation. The replay verifies the new excerpt, citation ID, and
unchanged document structure. The native test checks each excerpt against its
primary cited source. The historical Agent CLI capture remains an explicit
test fixture.
[Baseline](evidence/verification/round27-self-explanation-baseline.log.gz) ·
[Updated replay](evidence/verification/round27-self-explanation-green.log)

## Generated procedure-census fixture

On `9b544e8a5`, Layered CI ran 1,250 tests in its second JavaScript group.
The procedure-census freshness test reported that the fixture still listed
the two earlier fetch requests and omitted the new English project lookup.
The generator reads the specification corpus and now emits 122 prompts,
including the exact issue URL and the unregistered URL. The local baseline
reproduced the mismatch, and all eight census tests passed after regeneration.
[Hosted failure](https://github.com/link-assistant/formal-ai/actions/runs/45025294500/job/137219097157) ·
[Full workflow log](evidence/verification/round27-hosted-45025294500.log.gz) ·
[Job log, line 2722](evidence/verification/round27-hosted-45025294500-job-137219097157.log.gz) ·
[Baseline](evidence/verification/round27-census-baseline.log.gz) ·
[Regenerated replay](evidence/verification/round27-census-green.log.gz)

## Native recurrence with an empty source cache

The native job at `9b544e8a5` passed type-checking, the Rust behavior suites,
the cold runtime smoke, and the prompt-data replays. Its recurrence replay
then returned an empty string for a cold function request. The synthesis
trace records an unverified publisher recurrence. The same replay passed
with explicit publisher captures, including the three factorial locales and
the independent Fibonacci request. The recurrence probe deliberately yields
to other synthesis methods when it misses. This allowed the remaining
fallback to collapse to an empty answer. The solver now records the miss
and renders a localized source gap only if the ordinary solver and procedure
selection leave the answer empty. The regression checks that the gap is
visible and that no function is returned from the empty cache.

The job reached its 20-minute budget during the same replay. Cargo had built
the CLI twice with different optimization levels before running the serial
requests. Both Rust suites now share the optimized build. The experiment at
`8ca66ab3e` ran lint and both Cargo steps in under eight minutes. The second
Cargo step took six seconds. It passed the optimized executable to a separate
runner, but the existing policy confines the focused PR checks to one job with
no uploads. The workflow keeps the complete replay list on the compiler's
runner with the original time limit and the shared profile.
[Hosted job](https://github.com/link-assistant/formal-ai/actions/runs/45025294658/job/137220672532) ·
[Job log, lines 2877 and 3061](evidence/verification/round27-hosted-45025294658-job-137220672532.log.gz) ·
[Timeout annotation](evidence/verification/round27-hosted-45025294658-job-137220672532-annotations.json) ·
[Full workflow log](evidence/verification/round27-hosted-45025294658.log.gz) ·
[Shared-profile timing](evidence/verification/round28-native-timing.json) ·
[Build policy baseline](evidence/verification/round28-build-policy-red.log.gz) ·
[Build policy verification](evidence/verification/round28-build-policy-green.log.gz)

The fast source check at `8ca66ab3e` rejected the first fallback because the
synthesis handler grew from 198 to 212 lines. The existing skill-gap module
now owns the localized response. The synthesis handler records the miss and
reuses its existing response path for normal synthesis failures. Factoring
the research-trail formatting reduced the audited handler to 196 lines.
The local boundary check passed with both aggregate ceilings reduced by two.
[Hosted source check](https://github.com/link-assistant/formal-ai/actions/runs/45029139736/job/137228618261) ·
[Job log, lines 761–765](evidence/verification/round28-hosted-45029139736-job-137228618261.log.gz) ·
[Boundary baseline](evidence/verification/round28-minimal-core-baseline.log.gz) ·
[Boundary verification](evidence/verification/round28-minimal-core-green.log.gz)

Layered CI also detected that `solver.rs` changed after the governed test
fixture was generated. Regenerating the complete witness with the reviewed
reader changed 19 references to the solver source digest. The regenerated
inventory has zero remaining differences against a fresh reading, and the
two fixture-integrity tests and 34 complete-witness tests passed locally.
[Layered workflow log, lines 4263 and 6449](evidence/verification/round28-hosted-45029139075.log.gz) ·
[Witness comparison and checks](evidence/verification/round28-witness.log.gz)

## Native program input clarification

The same native replay passed the cold recurrence and the sourced recurrence
cases before reaching the program operand regression. A taskless Bash request
returned only the follow-on instructions. A probe against the exact CI
executable also reproduced empty replies for a missing factorial operand and
an unspecified FizzBuzz range, and a truncated reply for text reversal. The
solver had identified the missing caller inputs, but the question-necessity
filter classified the localized questions as factual because they lacked a
classification cue. The solver now marks these replies as requests for a
required input. They use the existing search trace and question budget.
The expanded HTTP regression checks that the operand and range questions
survive and leave execution pending.
[Replay job](https://github.com/link-assistant/formal-ai/actions/runs/45029139736/job/137230861025) ·
[Job log, lines 1309–1326](evidence/verification/round28-native-replay-137230861025.log.gz) ·
[Local reproduction](evidence/verification/round28-native-operand-red.log.gz) ·
[Probe and all replies](evidence/verification/round28-native-operand-probe.log.gz) ·
[Pipeline log](evidence/verification/round28-hosted-45029139736.log.gz)

The failure skipped the later hosted reporting and gateway steps. Running
those against the downloaded `8ca66ab3e` executable passed the Codex menu and
context replay, the gateway authorization and receipt test, and all 28 tests
in the released client report suite.
[Codex replay](evidence/verification/round28-native-codex.log.gz) ·
[Gateway replay](evidence/verification/round28-native-gateway.log.gz) ·
[Client replay](evidence/verification/round28-native-client-reports.log.gz)
