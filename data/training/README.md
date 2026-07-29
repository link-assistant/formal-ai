# Training and Distillation Sources

Formal AI currently has no approved neural training or distillation source.
This directory is the only permitted intake boundary for future
parameter-updating data:

- `source-registry.json` is the machine-readable approval ledger.
- `artifacts/` is the only permitted location for approved training payloads.
- `docs/legal/source-review.md` is the review form.
- `LEGAL-COMPLIANCE.md` is the governing policy.

The registry intentionally starts with an empty `sources` array and
`current_state: "no-approved-training-sources"`. CI compares every file below
`artifacts/` with the registry. It will fail closed if an artifact is
unregistered, an entry is incomplete, or its decision is not `approved`.

An entry must describe one immutable source and include all fields exercised by
`tests/unit/docs_requirements_issue_834.rs`. Put dated license and terms
evidence under the issue's `docs/case-studies/.../raw-data/` directory, when
redistribution is permitted; otherwise record a non-secret digest and review
location. Do not commit confidential contracts.

The Nemotron material under `data/benchmarks/` is not a training source. It is a
compact, provenance-recorded evaluation fixture. Benchmarks, runtime retrieval
data, caches, and community issue examples cannot be copied here or used to
update parameters without a fresh source review and approved registry entry.

Provider filters and metadata do not replace review. Pin the exact model,
version, provider route, terms date, permitted use, artifact hashes, and
downstream obligations.
