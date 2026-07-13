# Issue 482 Online Research

Research date: 2026-07-08.

## Primary Upstream Sources

- NVIDIA Nemotron 3 Ultra release page:
  <https://research.nvidia.com/labs/nemotron/Nemotron-3-Ultra/>
  - Published June 4, 2026.
  - Describes Nemotron 3 Ultra as a family with 550B total / 55B active
    parameters and says the release includes checkpoints plus the datasets used
    for training.
  - Links to pretraining code, legal, specialized, and post-training data
    collections.
- NVIDIA developer blog:
  <https://developer.nvidia.com/blog/nvidia-nemotron-3-ultra-models-and-datasets-accelerate-custom-ai-development/>
  - Describes the Ultra refresh as adding synthetic legal data, synthesized
    wiki-based data, refreshed GitHub/code data, SFT samples, RL tasks, and RL
    environments.
- Nemotron 3 Ultra model card:
  <https://huggingface.co/nvidia/Nemotron-3-Ultra-253B-v1>
  - Records OpenMDW-1.1 model terms, a pretraining data cutoff around September
    2025, and a post-training cutoff around May 2026.
  - Points to the Nemotron pre-training and post-training dataset families as
    major portions of the training corpus.

## Dataset Pages Checked

- Legal pretraining shard:
  <https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1>
  - Hugging Face API snapshot:
    `docs/case-studies/issue-482/raw-data/hf-nemotron-legal-api.json`
  - Dataset revision sampled here:
    `3d91d58a5c0c46fe9944300ec46719f97a385b13`
  - Dataset card license: `cc-by-4.0`.
  - Datasets-server split snapshot:
    `docs/case-studies/issue-482/raw-data/hf-nemotron-legal-splits.json`
  - Selected for the committed benchmark because both the card and sampled rows
    carry `CC-BY-4.0`, matching the repository benchmark policy.
- Specialized pretraining shard:
  <https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Specialized-v1.2>
  - Hugging Face API snapshot:
    `docs/case-studies/issue-482/raw-data/hf-nemotron-specialized-api.json`
  - Dataset revision observed:
    `807afc1fa65c441d46ebc7d9b95295a35499a527`
  - Dataset card lists both `cc-by-4.0` and `cc-by-2.0`. The sampler therefore
    rejects rows that do not have an explicit `cc-by-4.0` row license. The
    committed fixture does not import specialized rows.
- Dataset sample page:
  <https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Dataset-sample>
  - Snapshot:
    `docs/case-studies/issue-482/raw-data/hf-nemotron-sample-api.json`
  - Useful for schema discovery, but not selected for the committed fixture
    because the repository benchmark convention prefers explicit permissive
    source licensing on imported payload slices.

## Sampling Method

The sampler is `scripts/sample-nemotron-training-data.py`. It:

- uses Hugging Face datasets-server `rows` endpoint with `length=1`;
- discovers row counts through the same endpoint;
- selects deterministic random offsets from the seed `issue-482`;
- emits only compact excerpts, text length, SHA-256 digests, row IDs, config
  names, and provenance URLs;
- never downloads `.parquet` files or full splits;
- emits only rows whose row-level license is `CC-BY-4.0`.

Generated sample artifact:
`docs/case-studies/issue-482/raw-data/nemotron-random-samples.json`.

Executable fixture:
`data/benchmarks/nemotron-training-samples.lino`.

Ratchet tests:
`tests/unit/specification/nemotron_training_samples.rs`.
