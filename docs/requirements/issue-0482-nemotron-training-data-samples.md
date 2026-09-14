## Issue #482 Nemotron Training-Data Samples

Issue [#482](https://github.com/link-assistant/formal-ai/issues/482) asks to
use NVIDIA Nemotron 3 Ultra training data to add tests, without using the model
and without downloading the full dataset. PR
[#639](https://github.com/link-assistant/formal-ai/pull/639) adds a
license-aware sampler, a compact ten-row benchmark fixture, and a documentation
case study under `docs/case-studies/issue-482`.

| ID | Requirement | Status |
| --- | --- | --- |
| R435 | Use Nemotron 3 Ultra training data, not the released model weights or another LLM inference path. | Implemented by `scripts/sample-nemotron-training-data.py`, which samples Hugging Face dataset rows and records no model call. |
| R436 | Avoid downloading full upstream datasets. | Implemented by the sampler's datasets-server `rows` requests with `length=1`; covered by `issue_482_nemotron_training_sample_fixture_is_well_formed` and `issue_482_nemotron_training_ingestion_ratchet_passes_all_samples`. |
| R437 | Add ten random samples from the training data. | Implemented by `docs/case-studies/issue-482/raw-data/nemotron-random-samples.json` and `data/benchmarks/nemotron-training-samples.lino`, generated from seed `issue-482`. |
| R438 | Preserve row provenance so more samples can be fetched later. | Implemented by recording dataset, config, split, row offset, row UUID, revision, license, digest, and row URL for every sample. |
| R439 | Keep imported benchmark data within the repository's permissive-license policy. | Implemented by selecting the Nemotron Legal v1 shard and rejecting sampled rows without explicit `CC-BY-4.0` row licenses. |
| R440 | Turn the samples into executable tests. | Implemented by `tests/unit/specification/nemotron_training_samples.rs`, which validates Links Notation syntax, fixture/raw JSON consistency, no-full-download policy, sample diversity, and a 10/10 ingestion floor. |
| R441 | Index the new benchmark in the central catalog and license provenance. | Implemented by updates to `docs/benchmarks.md` and `data/benchmarks/LICENSES.md`. |
| R442 | Preserve issue data, online research, and solution planning under the required case-study directory. | Implemented by `docs/case-studies/issue-482/README.md`, `requirements.md`, `solution-plan.md`, and `raw-data/`. |
| R443 | Be explicit that this PR adds a training-data ingestion ratchet, not arbitrary legal-domain answering. | Implemented by the issue #482 README and solution plan; future legal QA/classification solving is listed as expansion work. |
| R444 | Protect the issue #482 documentation contract with automated traceability. | Implemented by `tests/unit/docs_requirements_issue_482.rs`, wired through `tests/unit/mod.rs`. |
