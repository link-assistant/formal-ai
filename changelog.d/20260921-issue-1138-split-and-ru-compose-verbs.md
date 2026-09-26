Fixed: Russian compose verbs are the compose signal, not the removed nouns.
0bc8d42e2 stopped retrieval theft by deleting the "материал"/"подробн" surfaces
from act_compose, which also silenced two held-out compose paraphrases
(ru_compose_03/07) whose only signal had been the noun. The lexeme now carries
the imperative verbs "наброса" and "составь" — exact counterparts of the
existing en surfaces "draft" and "put together" — so composition prompts route
by their act verb while "Найди подробные сведения…"-style retrieval prompts
keep reaching web_search through their own verbs (420/420 held-out cases).

Fixed: the tests-as-docs allowlist covers the fifteen loose-only behavioural
tests that the closure-audit step had masked (place_timezone, issue_435,
issue_595, translation_via_links, prompt_variations, intent_phrase_migration,
extended definition), regenerated with the gate's own `--write` migration.
