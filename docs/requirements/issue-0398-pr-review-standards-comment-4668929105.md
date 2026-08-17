## Issue #398 PR Review Standards (comment 4668929105)

The 2026-06-10 review accepted the corrected count (478 defined meanings, two
definition syntaxes) and the backbone `reference_closure.rs` gate, but required
that closure be widened from the structured backbone to **every** value token,
and that the agreed multi-source `view` infrastructure be built and
CI-enforced. These supersede the partial-grounding posture of R282 for the
scope they cover.

| ID | Requirement | Status |
| --- | --- | --- |
| R284 | Closure must be **total**: every non-keyword, non-quoted value token anywhere in `data/seed/**.lino` must resolve to a defined meaning (either syntax), a grounded `Q…/L…/P…`/Wiktionary/WordNet source with a checked-in cache record, or an override. CI must fail naming every unresolved token and must not pass until the count is 0; the backbone closure stays as a stricter subset. | Implemented by `scripts/audit-total-closure.py` (single-source resolver, `--json`/`--candidates`), `scripts/close-total.py` (idempotent migration defining every internal token as a meaning), and the gate `seed_has_total_reference_closure` in `tests/unit/total_closure.rs`. Audit reports 0 unresolved over 1,410 tokens. |
| R285 | WordNet (Open English WordNet 2024) must be mirrored in-repo (raw projection + `.lino`) and reachable by meanings, ingested by a preserved re-runnable script. | Implemented by `scripts/ground-wordnet.py` (offline OEWN 2024 import via `wn`) and 312 cached lemmas under `data/cache/wordnet/en/`; presence enforced by `wordnet_cache_is_present_and_used`. |
| R286 | A `data/view/` merge layer must exist with merged entities, deterministic `M-…` ids (same inputs → identical id), per-field provenance, and a merge that respects a threshold (same/different sense pairs merge/stay-separate). CI must fail if any piece is missing or not working. | Implemented by `scripts/build-views.py` (536 entities, `M-<sha1[:12]>` ids, per-sense provenance, Jaccard ≥ 0.5 merge, `--check`/`--selftest`) and the gates `multi_source_view_is_present_and_consistent` / `view_layer_has_real_multi_source_entities`. |
| R287 | `data/seed/sources-registry.lino` must list every ingested source with an API endpoint and a permissive license; CI must fail on an unlisted source that has a populated cache. | Implemented by `data/seed/sources-registry.lino` (Wikidata, Wiktionary, WordNet, Wikipedia) and `sources_registry_lists_every_ingested_source`. |
| R288 | All of the above must be delivered in this single PR through preserved, re-runnable scripts that double as migrations/loaders, and the PR must not be marked ready-to-merge while any token is undefined or any required infrastructure is absent. | Implemented: every mass action is a checked-in script (`audit-total-closure.py`, `ground-wordnet.py`, `ground-wiktionary.py`, `close-total.py`, `build-views.py`); the closure and infrastructure gates above keep the PR red until each requirement holds. |
