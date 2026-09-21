---
bump: patch
---

### Fixed

- A learning directive that names a URL ("learn from … at
  trends.google.com/…", "обратясь сюда ты узнаешь …", "यहाँ से सीख
  सकते हो …", "在这里了解…") now routes to `learn_from_source` instead of the
  generic web fetch (issue #499). The act vocabulary gains a ninth act,
  *learn* — narrower than *retrieve* because the request asks the engine to
  adopt knowledge from a declared source rather than fetch the URL once — with
  surfaces in `data/seed/meanings-acts.lino` and a `(url, learn, web)` row in
  the capability table. The handler stays gated on the seed-declared
  learnable-source registry, so a directive the registry declines falls
  through to the specialized walk unchanged, and cue-less prompts ("Open
  a bare URL") keep the fetch route.
