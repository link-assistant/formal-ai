# Issue 498 Solution Plans

## Trends Snapshot Converter

Use the Google Trends RSS feed as the smallest reliable automation point for the
first implementation:

1. Fetch `https://trends.google.com/trending/rss?geo=US&hl=ru`.
2. Parse `<item>` records into ranked topics with title, approximate traffic,
   publication date, and attached news references.
3. Render the parsed snapshot as Links Notation under
   `data/seed/google-trends-snapshot.lino`.
4. Keep live fetches out of tests; CI consumes the checked-in seed.

The official Trends API alpha is recorded as future prior art, but the first
implementation should not depend on limited-access credentials.

## Multilingual Prompt Expansion

For each top 10 topic, generate two request variations per supported language:

- `tell_me_about`: a direct topic request.
- `trends_context`: a request that names Google Trends context.

The expansion iterates over `supported_languages()` so the test fails if a new
supported language is added without a prompt template. The current supported
set is English, Russian, Hindi, and Chinese.

## Answer Stream

Do not create a special answer generator for search trends. The catalog should
reuse the existing engine:

1. Generate a prompt variant.
2. Call `FormalAiEngine::answer(prompt)`.
3. Store the prompt, language, variation, intent, confidence, answer text, and
   evidence links.

This makes Google Trends a regression surface for the current Formal AI answer
path. If a trend is unknown or underanswered, that limitation becomes visible in
the catalog.

## Agentic Recipe

Add a dedicated `google_trends_catalog` agentic recipe:

1. Route Google Trends / trending-search catalog requests without colliding with
   the question-catalog recipe.
2. Render the answered catalog as Links Notation.
3. Have the planner drive `write_file -> run_command -> final`.
4. Verify the file through an allowlisted compact `python3` command that prints
   the line count and catalog header.
5. Commit the generated Agent CLI session and assert a fresh run matches it
   byte-for-byte.

This follows the same reproducibility standard as the Issue #527 question
catalog while keeping the live Trends source isolated to the refresh command.
