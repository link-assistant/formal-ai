# Issue 571 Case Study

## Collected Data

Raw GitHub and reproduction evidence is preserved under `raw-data/`:

- `issue-571.json` and `issue-571-comments.json`
- `pr-618.json`, `pr-618-conversation-comments.json`,
  `pr-618-review-comments.json`, and `pr-618-reviews.json`
- `repro-before.log` and `repro-after.log`
- `failing-test-before-fix.log`, `focused-test-after-fix.log`, and
  `web-requests-test.log`
- `closure-audit.json`

## Requirement

Issue #571 reports the Russian prompt:

```text
есть ли скидка если брать подписки $200 Claude MAX и ChatGPT Pro не помесячно, а на год?
```

The requested behavior is not a hardcoded answer about those two products. The
maintainer comment asks for support across the whole repository and roadmap, so
the solution must handle the broader class of source-backed commercial
subscription, pricing, billing-period, and discount questions.

## Reproduction

Before the fix:

```sh
cargo run --bin formal-ai -- chat --prompt 'есть ли скидка если брать подписки $200 Claude MAX и ChatGPT Pro не помесячно, а на год?'
```

The solver returned the Russian `unknown` fallback and said it could not
determine the prompt from local memory, public knowledge cache, or available
sources.

The regression test added for this issue first failed the same way for the exact
Russian prompt, proving the bug was a routing gap rather than a presentation
issue.

## Root Cause

The web-search recognizer had a seed-driven `implicit_research_question` path
for prompts that combine:

- a research question opener;
- either a research modifier or an evidence-domain plus evaluation-domain pair.

The reported prompt missed both parts of that seed vocabulary. Russian yes/no
question forms such as `есть ли ...` were not research-question openers, and
commercial subscription/pricing terms such as `подписки`, `год`, and `скидка`
were not present in the evidence/evaluation domains. With no handler claiming
the prompt, it fell through to unknown reasoning.

The deeper problem the maintainer raised is that this path is still
*vocabulary-driven*: it can only ever recognise the specific topic words that
have been seeded. It therefore cannot, on its own, cover the open-ended *class*
the issue asks for — any question about a real-world product, service, or
organisation whose current facts live on the public web.

## Implementation

The fix has two complementary layers.

1. **Seed vocabulary (specific example).** `data/seed/meanings-web-research.lino`
   gains yes/no research-question openers across English, Russian, Hindi, and
   Chinese, plus commercial subscription/pricing/billing-period terms as
   `research_evidence_domain` and discount/price/cost/deal terms as
   `research_evaluation_domain`. This routes the exact reported prompt through
   the existing generic recognizer without a product-specific answer table.

2. **Reasoning rule (entire class).** `extract_externally_verifiable_question`
   in `src/solver_handlers/web_search_intent.rs` closes the generalisation gap by
   reasoning about the *referent* rather than the topic vocabulary. It routes a
   prompt to web research when three structural conditions all hold:

   - the prompt is *interrogative* — it opens with a seeded question opener (any
     language) or ends with a question mark (`?` / fullwidth `？`);
   - it names a *referential external entity* — a Latin token written with
     interior capitalisation (`ChatGPT`, `OpenAI`, `GitHub`, `iPhone`,
     `TypeScript`), the orthographic signature of an engineered brand/product
     name; and
   - the solver cannot answer it locally — it is neither a seeded concept lookup
     nor a self-introduction / capability / non-referential-subject question.

   Because interior capitalisation is a property of the Latin brand token itself,
   the rule fires identically whether that token sits in English, Cyrillic,
   Devanagari, or CJK context, and across any topic (pricing, release dates,
   hardware specs, features). All-caps acronyms (`BSD`, `ML`, `USD`), plain
   capitalised proper nouns (`Claude`, `Tesla`, `Wikipedia`), and Title-Cased
   phrases (`Hive Mind`) deliberately do not match, so concept-lookup, coding,
   and unknown-reasoning prompts keep their own handlers.

The reasoning rule is a pure structural test with no per-product or per-language
word list to maintain, so the Russian annual-discount prompt is handled as one
instance of the general class rather than a special case.

## Verification

Focused regression:

```sh
cargo test --test unit commercial_subscription_discount_questions_route_to_web_search_handler -- --nocapture
```

Broader web-search routing group:

```sh
cargo test --test unit web_requests -- --nocapture
```

After the fix, the exact CLI prompt returns:

```text
Распознан исследовательский вопрос для `скидка если брать подписки 200 claude max и chatgpt pro не помесячно а на год`.
```

The evidence includes `web_search:query_kind:implicit_research_question`, so
the solver no longer reports the prompt as unknown.
