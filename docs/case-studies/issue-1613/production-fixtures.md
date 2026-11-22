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
and concept fallback returns registered reference text. Brainstorm categories
now choose composition operations, which combine the supplied topic with
generic prompts for designs. Naming requests use the existing morphological
composer. Missing topics elicit a clarification. The browser tests cover
distinct ordinary topics as well as the previous promotion routes.
[Summary replay](evidence/verification/round26-summary-after.log) ·
[Handler replay](evidence/verification/round26-final-issue-0918-handler-rules-batch.log) ·
[Claim routing replay](evidence/verification/round26-final-issue-1175-claim-routing.log)

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
