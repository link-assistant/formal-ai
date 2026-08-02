# Issue 482 Requirements

Issue: <https://github.com/link-assistant/formal-ai/issues/482>

| ID | Requirement | Acceptance |
| --- | --- | --- |
| R482-01 | Use NVIDIA Nemotron 3 Ultra training data, not the released model weights or an LLM call. | The implementation samples dataset rows from Hugging Face datasets-server and records no model inference dependency. |
| R482-02 | Do not download the full dataset. | The sampler uses the `rows` endpoint with `length=1`; tests reject parquet/full-split provenance URLs. |
| R482-03 | Add 10 random samples. | `data/benchmarks/nemotron-training-samples.lino` contains exactly ten deterministic random row samples from seed `issue-482`. |
| R482-04 | Preserve enough provenance to fetch more samples later. | `scripts/sample-nemotron-training-data.py` records dataset, config, split, row offset, revision, row UUID, and row URL. |
| R482-05 | Keep the imported benchmark slice license-compatible with repository policy. | The committed fixture uses only rows with explicit `CC-BY-4.0` row licenses. |
| R482-06 | Turn the samples into tests that can improve the AI system over time. | `tests/unit/specification/nemotron_training_samples.rs` creates a 10/10 ingestion ratchet over row metadata, excerpt previews, digests, and no-full-download policy. |
| R482-07 | Collect issue data and online facts under `docs/case-studies/issue-482`. | The case-study directory preserves issue/PR snapshots, code searches, related PR searches, Hugging Face API captures, and online research notes. |
| R482-08 | Consider existing repository benchmark conventions. | The fixture is indexed in `docs/benchmarks.md` and `data/benchmarks/LICENSES.md`, following prior benchmark ratchets. |
| R482-09 | Be honest about current Formal AI limitations. | The case study records that this PR adds a training-data ingestion benchmark, not a legal-domain question-answering solver. |
| R482-10 | Land the work in PR #639. | PR #639 is updated from the prepared branch `issue-482-aaf2da83253d`. |
