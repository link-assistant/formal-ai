# Issue 482 Case Study

Issue #482 asks Formal AI to use NVIDIA Nemotron 3 Ultra training data to add
tests, without using the model and without downloading the full dataset.

## Collected Data

| Artifact | Purpose |
| --- | --- |
| `raw-data/issue-482.json` | Original issue snapshot. |
| `raw-data/issue-482-comments.json` | Issue comments; empty at capture time. |
| `raw-data/pr-639*.json` | Prepared PR metadata and review/comment streams. |
| `raw-data/code-search-nemotron.txt` | GitHub code search for existing Nemotron usage. |
| `raw-data/code-search-training-data.txt` | GitHub code search for existing training-data patterns. |
| `raw-data/recent-merged-benchmark-prs.json` | Recent benchmark PR style references. |
| `raw-data/recent-merged-case-study-prs.json` | Recent case-study PR style references. |
| `raw-data/hf-nemotron-*.json` | Hugging Face dataset metadata and split captures. |
| `raw-data/nemotron-random-samples.json` | Compact 10-row deterministic random sample artifact. |
| `raw-data/online-research.md` | Upstream source notes and dataset-selection rationale. |

## Requirements

The issue is decomposed in [`requirements.md`](requirements.md). The core
constraints are:

- no Nemotron model/LLM use;
- no full dataset download;
- ten random training-data samples;
- reusable script for more samples;
- tests that make the sample path part of CI;
- all evidence under `docs/case-studies/issue-482`.

## Root Cause

Before this PR, Formal AI had benchmark fixtures and provenance rules, but no
Nemotron training-data ingestion path. The repository also had no safe sampler
for Hugging Face datasets-server rows, so adding real Nemotron samples without
accidentally importing a full corpus required a new small tool and a new
fixture-level ratchet.

## Implemented Design

- `scripts/sample-nemotron-training-data.py` samples individual rows through the
  Hugging Face datasets-server `rows` endpoint with `length=1`.
- `docs/case-studies/issue-482/raw-data/nemotron-random-samples.json` preserves
  ten deterministic samples from seed `issue-482`.
- `data/benchmarks/nemotron-training-samples.lino` converts those samples into
  an executable benchmark fixture.
- `tests/unit/specification/nemotron_training_samples.rs` validates the fixture,
  matches it back to the raw sampler output, and enforces a 10/10 ingestion
  ratchet.
- `docs/benchmarks.md` and `data/benchmarks/LICENSES.md` index the new suite.

## Prior Art And Existing Components

The implementation follows the repository's benchmark pattern from issues #304,
#317, #362, #408, and #444:

- small committed slices or generated local-profile cases;
- provenance recorded in `data/benchmarks/LICENSES.md`;
- no full upstream corpora vendored;
- `minimum_pass_count` ratchets in Rust tests;
- benchmark catalog entry in `docs/benchmarks.md`.

## Verification

Targeted check:

```sh
cargo test --test unit issue_482_nemotron_training -- --nocapture
```

Expected result: 3 tests pass.

The tests deliberately check ingestion, provenance, licensing, and no-full-data
policy. They do not claim that the current symbolic solver can answer arbitrary
legal questions from the sampled training rows.
