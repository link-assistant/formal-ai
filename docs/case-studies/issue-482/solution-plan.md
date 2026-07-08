# Issue 482 Solution Plan

## Plan Considered

1. **Direct model use.** Rejected. The issue explicitly asks not to use the
   Nemotron model or any LLM inference path.
2. **Full dataset import.** Rejected. The repository benchmark convention avoids
   vendoring full upstream corpora, and the issue explicitly says not to download
   the full dataset.
3. **Small deterministic row sampler plus ingestion ratchet.** Chosen. This
   keeps the upstream data path real, reproducible, and license-aware while
   staying narrow enough for CI.

## Delivered Slice

- Preserve issue, PR, code-search, related-PR, and upstream Hugging Face API
  snapshots under `docs/case-studies/issue-482/raw-data/`.
- Add `scripts/sample-nemotron-training-data.py`, a deterministic sampler that
  fetches individual Hugging Face datasets-server rows with `length=1`.
- Commit `docs/case-studies/issue-482/raw-data/nemotron-random-samples.json`, a
  compact 10-row sample artifact from the Nemotron Legal v1 training shard.
- Add `data/benchmarks/nemotron-training-samples.lino`, the executable fixture
  derived from the sample artifact.
- Add `tests/unit/specification/nemotron_training_samples.rs`, which checks
  Links Notation validity, sampler/fixture consistency, license policy,
  no-full-download provenance, digest presence, config diversity, and a 10/10
  ingestion floor.
- Index the new suite in `docs/benchmarks.md` and
  `data/benchmarks/LICENSES.md`.

## Why The Fixture Is Legal-Only

Nemotron 3 Ultra links multiple training-data shards. The legal shard carries a
dataset-level `cc-by-4.0` license, and sampled rows carry explicit `CC-BY-4.0`
licenses. The specialized shard's card lists both `cc-by-4.0` and `cc-by-2.0`,
so the committed fixture avoids importing it until a future sampler mode can
prove row-by-row compatibility for a larger mixed-license set.

## Future Expansion

- Add a network/ignored test that refreshes samples on demand into
  `target/formal-ai-benchmarks` for broader local experiments.
- Add legal-domain symbolic parsing cases once the solver has deterministic
  legal QA or classification capabilities worth ratcheting.
- Teach the Agent CLI a reusable benchmark-ingestion recipe so future dataset
  imports can be driven through Formal AI's own agentic surface instead of this
  manually orchestrated first slice.
