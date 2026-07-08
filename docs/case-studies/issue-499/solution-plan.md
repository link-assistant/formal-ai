# Issue 499 Solution Plan

## RSS Snapshot

Collect the live Google Trends data as an explicit raw-data step, not inside CI.
For this PR the source is:

```sh
curl -L -D docs/case-studies/issue-499/raw-data/google-trends-us-rss-headers.txt \
  -o docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml \
  "https://trends.google.com/trending/rss?geo=US"
```

The committed snapshot is the reproducible input. A future collector can refresh
it on demand, but the tests should never fail because Google Trends changed
between two CI runs.

## Prompt Catalog

Parse each RSS `<item>` and its `ht:*` metadata into a ranked trend record. Take
the first ten rows and render:

- one `google_trends_prompt_suite` record;
- ten `google_trends_topic` records;
- forty `google_trends_prompt_case` records, one for each supported language and
  topic pair.

The prompts are self-authored templates:

- `en`: `What is {topic}?`
- `ru`: `Что такое {topic}?`
- `hi`: `{topic} क्या है?`
- `zh`: `{topic} 是什么?`

## Agentic Recipe

Expose the same deterministic renderer through
`src/agentic_coding/trend_prompt_catalog.rs`. The planner route is intentionally
small:

```text
write_file(data/benchmarks/google-trends-top10-suite.lino) -> run_command(cat ...) -> final
```

That mirrors the issue #527 generated question catalog and produces a pinned
Agent CLI JSON session for review.

## Regression Tests

Cover the conversion at every boundary:

- parse the committed RSS snapshot and assert the top-ten source topics are
  present;
- compare the committed `.lino` fixture byte-for-byte with a fresh render;
- assert every top-ten topic has English, Russian, Hindi, and Chinese prompt
  cases;
- run every generated prompt through the offline solver and assert it produces a
  usable response object;
- route and drive the Agent CLI recipe end to end;
- assert the case-study and benchmark-catalog documentation remain traceable.

## Follow-Ups

This PR intentionally stops at a deterministic collector and benchmark fixture.
Later auto-learning work can add a scheduled refresh job, human approval for
promoting useful trend answers into seed data, and optional adapters for the
official Google Trends API alpha when it becomes generally available.
