# Issue #872 Case Study: The App Store Request With Constraints

Issue [#872](https://github.com/link-assistant/formal-ai/issues/872).
Reported from the agentic CLI (0.304.0, session
`ses_06199e7baffeHQKxVESGxaSzTQ`) on 2026-07-26; case study written on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

> U: игры для малышей с открытым исходным кодом (абсолютно и полностью
>    бесплатные) в App Store (iOS)
> A: Я не смог определить `игры для малышей …` … (unknown fallback, with
>    the teach-me-a-rule template)

The request has no request verb — it is a bare noun phrase with a
marketplace and two qualifiers. Every cue-based router missed it, and the
unknown fallback answered with instructions for teaching a rule, which is
not what a parent browsing for toddler apps needs.

## Requirements

1. A bare noun phrase with a marketplace phrase routes as a shopping
   request (no verb required).
2. The qualifiers are constraints, stated in the answer: open source,
   completely free (and their combination).
3. Constraint-specific advice: stores do not enforce the open-source
   label; "fully free" must be checked against in-app purchases and ads.
4. Kids-app advice: age band and requested permissions, offline-capable
   listings to avoid ads served to children.
5. The query keeps the user's own words (the Russian surface), not a
   catalogue slug.
6. No marketplace, no claim — the handler declines.

## Solution (this pull request, extending the #800 handler)

- `data/seed/product-search-cues.lino` already carries the
  `app_store_ios` marketplace (deep link
  `apple.com/us/search/{query}`), the `free`/`fully_free`/`open_source`
  constraint rows with their filter advice, and the `toddler_games`
  product noun with age-band/permissions advice.
- `rust/src/solver_handlers/product_search.rs` gained the structural
  trigger: the handler fires on an intent cue **or** the shopping shape
  (marketplace phrase + product noun) — issue #872's verbless prompt —
  and composes the query from the noun's matched surface phrase plus
  brand/model terms instead of the catalogue slug.
- `rust/tests/unit/issue_872_app_store_search.rs` pins the exact reported
  string, the App Store link, both constraints, the toddler advice, and
  the no-marketplace decline.

## Known components surveyed

- App Store web search (`apple.com/us/search/<query>?src=globalnav`) is a
  public GET surface; the store region in the path is a limitation noted
  for follow-up (a ru region link would serve the reporting user better;
  the seed's `link` row is the single edit point).
- F-Droid would be the natural open-source source for Android twins; a
  `marketplace` row can be added as data when requested.

## Integration site left for main

Same as #800: dispatch entry, precedence rank, `mod`/`pub use`, `lib.rs`
lift, seed-registry rows.

## Verification

- Automated: the four tests in
  `rust/tests/unit/issue_872_app_store_search.rs` (CI).
- Manual: replay the reported prompt with live fetch and open the link.
