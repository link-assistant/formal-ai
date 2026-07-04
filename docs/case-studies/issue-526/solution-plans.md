# Issue 526 Solution Plans

## R526-1 to R526-4: Natural-Language Round Trips

Use the existing `TranslationPipeline` rather than introducing a new evaluator.
The smallest complete slice is one meaning with surfaces in every supported
language. The apple meaning is already seeded in en, ru, hi, and zh, so it can
drive a full directed pair matrix:

1. Translate each source surface to each target language.
2. Assert the target surface is the expected target-language surface.
3. Translate that target surface back to the source language.
4. Assert the forward and backward `MeaningId` values are equal.
5. Assert the final surface equals the original source surface.

This catches loss in either direction and proves the path is source -> meta ->
target rather than a one-way display table.

## R526-5: Rust <-> JavaScript Code Meaning

Keep the code slice deliberately narrow and semantic:

1. Add a failing test for `fn add(a, b)` translated Rust -> JavaScript -> Rust.
2. Require evidence links for `language_from`, `language_to`, and `meaning`.
3. Normalize simple add-function implementations to one code meaning,
   `function:add:binary_sum`.
4. Render that meaning to JavaScript and Rust in `translate_program`.
5. Leave unknown code as explicit translation gaps.

## R526-6 to R526-8: Documentation And Governance

Update the root requirement matrix, vision, roadmap, architecture, and
contributor rules in the same PR. Add a docs guard test so future changes cannot
remove the round-trip contract or the case-study evidence silently.
