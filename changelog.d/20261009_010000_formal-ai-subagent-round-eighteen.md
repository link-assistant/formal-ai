### Changed

- PR #1188 round 18:
  - **Generalize, don't specialize (R1188-U1).** None of the test prompts in the solver and worker code remain there verbatim (23 → 0). Their words and answers moved to seed data:
    - the behavior-rule catalog: 49 `behavior_rule_*` intents in five languages (`multilingual-responses-behavior-rules.lino`), which both roots read by intent. The browser's list and detail answers now match the native ones byte for byte.
    - the rule-detail openings: a seeded `rule_detail_request` role.
    - the browser capability listings: the seeded `capabilities` responses, which fixes hi, zh and es.
    - the knowledge-export example prompts: `example` lines in `intent-routing.lino`.
    - the browser translation table: `translation_phrase` meanings (`meanings-translation-phrases.lino`).

    The specialization gate now also scans the app's `.jsx` files. It finds 17 pairs there, which are the ceiling now. Other ratchets fell:
    - hardcoded-language rows: 788 → 700;
    - JS worker literals: 645 → 445.

    `behavior_rules` is migrated to seed (R344: 58 migrated, 13 pending).
  - **G98 (destructive) fixed in both roots.** "Change 'a' to 'b' in the file f.txt" wrote `'b' in the`. The new text now ends before any article or function word that comes ahead of the target cue.
  - **Meaning language parity (LEXEMES):** the language-parity debt falls from 611 to 53 rows (from 910 at the start of the batch).
    - Spanish lexemes were added across the meaning files, plus ru, hi and zh where English stood alone.
    - The lexicon importer now takes the registry's partial languages (es) from the cached Wikidata labels as optional surfaces. It packs shards by line budget, and its offline reproduction regenerates the five import shards by rule.
    - Files that passed 1500 lines are split by category: number words, conversation summary, software requirements, language protocol and decomposition ladder.
    - Meanings that share a surface are ranked by languages, then by fewer surfaces, then by declaration order.
    - The round-trip measure divides by all content terms, so learning a word can no longer lower it. Term survival is 0.436.
    - The remaining gaps are deliberate, each with a reason: relational words that Spanish says with function words, and structural single-language records.
