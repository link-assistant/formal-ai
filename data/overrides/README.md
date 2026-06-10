# `data/overrides/` — grounding override layer

This directory sits **beside `data/cache/`** and mirrors its per-id structure
exactly:

```
data/cache/wikidata/entity/Q131560.lino      # what upstream returned
data/overrides/wikidata/entity/Q131560.lino  # human corrections to it
```

## Resolution order

Whenever the seed resolves an external-source record, the value a consumer sees
is computed as:

```
(checked-in cache record  OR  a live API fetch)   then   overrides
```

The override **decorates** the cache record — it never replaces the whole
record. `formal_ai::seed::resolve(cache, override)` applies the override's facts
on top of the cache projection: a `section / key value` fact in the override
wins over the same key in the cache, and a key the cache lacks is appended.

## What an override file looks like

An override repeats the top-level id, records **why it exists**, and lists only
the corrected or supplemented values:

```
Q131560
  reason "Wikidata carries no Hindi (hi) label for the KISS principle; the
          acronym is kept in Latin, matching the en and ru labels, so every
          supported language resolves a label."
  labels
    hi KISS
```

Rules (enforced by `tests/unit/overrides.rs`, which walks this whole tree):

1. **Real id.** The file path must map to an id that has a checked-in cache
   record under `data/cache/...`. Overrides decorate cached records; they do not
   invent ids.
2. **Recorded reason.** Every override must carry a non-empty `reason "..."`
   explaining why upstream is insufficient.
3. **Never redundant.** If the cache (after a refresh, or because upstream caught
   up) already carries an override's exact `section / key value`, the override is
   redundant and CI **fails** until it is removed. This guarantees the layer only
   ever holds genuine deltas from upstream and self-prunes as sources improve.

## Adding an override

Prefer fixing the source. Only add an override when upstream is genuinely
missing or wrong and cannot be corrected at the source in time. Record the
reason, keep the delta minimal, and delete the override once a cache refresh
makes it redundant — CI will remind you.
