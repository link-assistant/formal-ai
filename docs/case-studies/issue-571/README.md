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

The web-search recognizer already had a seed-driven `implicit_research_question`
path for prompts that combine:

- a research question opener;
- either a research modifier or an evidence-domain plus evaluation-domain pair.

The reported prompt missed both parts of that seed vocabulary. Russian yes/no
question forms such as `есть ли ...` were not research-question openers, and
commercial subscription/pricing terms such as `подписки`, `год`, and `скидка`
were not present in the evidence/evaluation domains. With no handler claiming
the prompt, it fell through to unknown reasoning.

## Implementation

The fix extends `data/seed/meanings-web-research.lino` instead of adding a
product-specific answer table:

- add yes/no research-question openers across English, Russian, Hindi, and
  Chinese;
- add commercial subscription/pricing/billing-period terms as
  `research_evidence_domain`;
- add discount/price/cost/deal terms as `research_evaluation_domain`.

The existing Rust and browser-worker logic reads these roles from the seed
lexicon, so the same generic recognizer now routes this class of questions to
the source-gathering web research plan.

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
