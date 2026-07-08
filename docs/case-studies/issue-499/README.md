# Issue 499 Case Study: Google Trends To Formal AI Requests

Issue [#499](https://github.com/link-assistant/formal-ai/issues/499) asks for
automated scripts and tools that turn Google Trends into Formal AI test-case
data: the top ten trending requests should become usable Formal AI requests,
with variations in every supported language, and the research should be
preserved under `docs/case-studies/issue-499`.

## 1. Collected Data

The raw capture lives in `raw-data/`:

- `issue-499.json` and `issue-499-comments.json` preserve the GitHub issue.
- `pr-641*.json` preserves PR discussion, review comments, and reviews.
- `recent-related-merged-prs.json` records related benchmark and catalog PRs.
- `google-trends-us-rss.xml` is the captured Google Trends US RSS snapshot from
  `https://trends.google.com/trending/rss?geo=US`.
- `google-trends-us-rss-headers.txt` records the HTTP response headers. The
  capture was made on `2026-07-08T20:29:32Z`.
- `manual-formal-ai-grok-4-5-before.log` and
  `manual-formal-ai-blue-jays-before.log` show that current trend terms were
  usable inputs but resolved to unknown local knowledge before this issue added
  a trend-derived prompt catalog.
- `online-research.md` records Google Trends, API, and third-party library
  research.

The captured RSS top ten were converted into
[`data/benchmarks/google-trends-top10-suite.lino`](../../../data/benchmarks/google-trends-top10-suite.lino).
That suite contains 10 trend topics and 40 Formal AI request cases: English,
Russian, Hindi, and Chinese prompts for each topic.

## 2. Requirements

The decomposed requirements are in [`requirements.md`](requirements.md) and the
root requirement table under "Issue #499 Google Trends Requirements". The key
acceptance points are:

- `R499-1`: preserve issue, PR, research, and source data.
- `R499-2`: provide an automated converter from Google Trends RSS snapshots.
- `R499-3`: convert the top ten trends into Formal AI request cases.
- `R499-4`: generate every supported language variation.
- `R499-5`: keep CI offline and deterministic.
- `R499-6`: prove the converter through the in-repo Agent CLI recipe.

## 3. Root Cause

Formal AI had benchmark fixtures and multilingual prompt matrices, but no
pipeline from a live trend source into reviewable request data. A prompt such as
`What is grok 4.5?` or `What is blue jays?` reached the solver as a normal
request, but there was no committed catalog connecting current public demand to
regression tests or future human-gated learning.

The second gap was reproducibility: Google Trends is live and changes often, so
CI cannot depend on a network query. The fix therefore separates data collection
from tests: capture RSS once, commit the snapshot as raw evidence, and generate
deterministic `.lino` cases from that snapshot.

## 4. Implemented Design

The implementation adds:

- `src/google_trends.rs`: parses Google Trends RSS items and renders a Links
  Notation prompt suite.
- `formal-ai google-trends`: converts a saved RSS file into a `.lino` fixture.
- `data/benchmarks/google-trends-top10-suite.lino`: the generated top-ten
  prompt catalog.
- `src/agentic_coding/trend_prompt_catalog.rs`: an Agent CLI recipe that writes
  and verifies the generated catalog.
- `docs/case-studies/issue-499/agent-cli-session-google-trends.json`: the
  pinned in-repo Agent CLI session proving the recipe.
- `tests/unit/issue_499_google_trends.rs`: parser, fixture, prompt usability,
  planner, and driver tests.
- `tests/unit/docs_requirements_issue_499.rs`: traceability coverage for the
  case study and benchmark catalog.

Regeneration command:

```sh
cargo run --bin formal-ai -- google-trends \
  --input docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml \
  --output data/benchmarks/google-trends-top10-suite.lino \
  --captured-at 2026-07-08T20:29:32Z
```

## 5. Prior Art And Existing Components

Local components reused:

- `docs/benchmarks.md` and `data/benchmarks/*.lino` established the benchmark
  fixture convention.
- Issue #527's question catalog established the generated document plus Agent
  CLI session pattern.
- Issue #444 and issue #408 established self-authored local prompt profiles with
  ratchet tests.

Online research found:

- Google Trends Trending Now exposes export options including RSS and CSV.
- Google Search Central documents Trending Now and Explore as sources for
  rising and top search interests.
- Google Trends Help documents geography and time filters for Trending Now.
- Google's Trends API alpha is official but limited and not appropriate as a
  required CI dependency yet.
- `pytrends` and commercial APIs can help future collectors, but they are not
  needed for the deterministic committed snapshot converter.

## 6. Verification

The intended verification commands are:

```sh
cargo test --test unit issue_499_google_trends -- --nocapture
cargo test --test unit docs_requirements_issue_499 -- --nocapture
cargo run --bin formal-ai -- google-trends \
  --input docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml \
  --output data/benchmarks/google-trends-top10-suite.lino \
  --captured-at 2026-07-08T20:29:32Z
cargo run --bin formal-ai -- agent \
  --task "Convert the captured Google Trends top ten RSS snapshot into Formal AI request cases, with one prompt variation in every supported language, and record the generated prompt catalog as Links Notation." \
  --session-json docs/case-studies/issue-499/agent-cli-session-google-trends.json
```
