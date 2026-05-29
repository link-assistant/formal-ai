# Reproduction dialog — Issue #341

Legend: `U` = user, `A` = agent. Captured from the issue body (version 0.149.0,
WASM worker, manual mode, locale `ru-RU`, `Asia/Yekaterinburg`).

````
U: Design a simple web scraper in Python that:
   1. Fetches a webpage
   2. Extracts all headings (h1, h2, h3)
   3. Counts word frequency
   4. Generates a markdown summary

   Then test it by scraping wikipedia.org and show me the top 10 most frequent words.

A (intent: agent_plan, reported): ## Agent plan (2 steps)
   1. Design a simple web scraper in Python that: ... (4 numbered requirements)
   2. test it by scraping wikipedia.org and show me the top 10 most frequent words.

   ### Step 1: Design a simple web scraper in Python that: ...
   software_project_request   (full lino meaning record, proposed plan, approval gates)
   Reply `approve plan` to generate the starter implementation, or describe what to change.

   ### Step 2: test it by scraping wikipedia.org and show me the top 10 most frequent words.
   I'm not sure how to respond to that yet. I cannot answer that from local Links
   Notation rules yet. To inspect what I can do, send `List behavior rules`, then
   `Show behavior rule unknown`. ...
````

## What is wrong

The agent-mode decomposer (`decomposeAgentTask` in `src/web/app.js`) splits the
prompt on the `Then` separator into two steps:

1. **Step 1** — "Design a simple web scraper in Python that: …" → handled well
   by `try_software_project_request`, which returns `software_project_plan`.
2. **Step 2** — "test it by scraping wikipedia.org and show me the top 10 most
   frequent words." → **misrouted**. In the deployed offline WASM worker it hit
   the **unknown-intent opener** ("I cannot answer that from local Links
   Notation rules yet"). Reproduced locally against the current `main`, the same
   step lands on a **`wikipedia` concept lookup** ("Wikipedia is a free,
   multilingual online encyclopedia …"). Both answers ignore that step 2 is a
   follow-up that wants to *test the scraper from step 1*.

## Local reproduction (current `main`, before fix)

`cargo run --example repro_issue_341` printed:

```
=== STEP 1 intent: software_project_plan ===
=== STEP 2 intent: concept_lookup ===
--- STEP 2 answer ---
Wikipedia (encyclopedia): Wikipedia is a free, multilingual online encyclopedia …
```

## After fix

```
=== STEP 1 intent: software_project_plan ===
=== STEP 2 intent: software_project_followup ===
--- STEP 2 answer ---
Recorded a verification follow-up for the scraper from the active plan. …
```
