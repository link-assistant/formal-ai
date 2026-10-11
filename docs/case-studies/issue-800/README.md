# Issue #800 Case Study: The Charger Search That Died in Research

Issue [#800](https://github.com/link-assistant/formal-ai/issues/800),
filed automatically from an agentic session on 2026-07-26; case study
written on the `qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

> U: Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in
> A: Research completed for мне зарядку для ноутбука acer aspire 3 a325 45
>    на amazon in, but the tool returned no content.

The request names everything a shopping search needs — marketplace
(`amazon.in`), product (`зарядка`, charger), brand and exact model
(`Acer Aspire 3 A325-45`) — but the router saw only a research-shaped
prompt and the research tool had nothing to return for it.

## Requirements

1. A shopping request with a marketplace composes the site-scoped search
   its terms describe (not a generic research pass).
2. The exact model code survives into the query (accessory shopping is
   model-exact; `A325-45` is not `A325`).
3. The answer is honest in offline mode: states nothing was fetched, shows
   the exact query link.
4. Buying-critical advice comes from data, per product noun (a charger:
   model, connector, wattage).
5. Requests without a marketplace phrase are declined, not guessed.

## Solution

- `data/seed/product-search-cues.lino` — marketplace records (host +
  deep-link template + trigger phrases, amazon.in / App Store / Google
  Play), request cues, qualifier constraints with filter advice, and
  product-noun advice rows. All vocabulary is data.
- `rust/src/solver_handlers/product_search.rs` — extracts the product
  terms (matched noun, model codes by shape — an uppercase
  letters+digits+hyphen token — and the brand words directly preceding
  them), the matched constraints, and the marketplace; percent-encodes the
  composed query; renders the localized template from
  `data/seed/multilingual-responses-product-search.lino` (en/ru/hi/zh/es)
  with the offline honesty statement.
- `rust/tests/unit/issue_800_product_search.rs` pins the exact reported
  string, the `amazon.in/s?k=` link shape, the model code in the query,
  the honesty marker, and the decline paths.

## Known components surveyed

- Marketplace deep-link shapes: `amazon.<tld>/s?k=<query>`,
  `apple.com/us/search/<query>`, `play.google.com/store/search?q=<query>`
  — all public GET surfaces, no affiliation param is appended.
- Existing `try_web_search` composes provider searches; the product path
  stays separate because the query is site-scoped, not provider-scoped,
  and the answer carries buying advice the generic search must not.

## Integration site left for main

`handle_product_search` needs its `HANDLER_FUNCTIONS` entry, precedence
rank after web-search intent, `mod`/`pub use` lines, `lib.rs` lift, and
seed-registry rows for `product-search-cues` +
`multilingual-responses-product-search` (+ mirrors already staged).

## Verification

- Automated: the four tests in `rust/tests/unit/issue_800_product_search.rs`
  (CI).
- Manual: replay the reported prompt with live fetch enabled and open the
  composed link.
