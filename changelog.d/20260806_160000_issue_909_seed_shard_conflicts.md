---
bump: patch
---

### Changed
- `data/seed/closure-generated-*.lino` shards are now content-addressed: each
  generated meaning is placed by `sha256(slug) % SHARD_COUNT` instead of filling
  shards in sorted order up to a line cap. Sequential fill made every shard depend
  on the size of everything sorted before it, so adding one token rewrote up to
  11 of 11 shards and `data/seed` conflicted in nearly every pull request; a new
  token now touches exactly one shard.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))
