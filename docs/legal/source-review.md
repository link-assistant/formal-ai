# Training Source Review

Complete this review before acquiring or generating a training or distillation
artifact. The reviewer must verify primary evidence; a model hub tag,
aggregator label, price, or provider filter is not enough. If any required
answer is unknown, the decision is `rejected` or `pending`, and no payload may
enter `data/training/artifacts/`.

## Identity and purpose

- Review date and reviewer:
- Source ID:
- Exact model/dataset name and model version or immutable revision:
- Source kind (dataset, model output, reasoning trace, preference data, other):
- Intended parameter-updating use:
- Intended output/model license, name, territories, and distribution:
- Artifact paths and SHA-256 digests:
- Evidence paths:

## Acquisition and applicable terms

- Upstream creator:
- Provider and exact provider route:
- Acquisition method and account/tier:
- Acquisition date:
- Upstream license URL:
- API/hosting Terms of Service URL:
- Terms checked date and terms snapshot or digest:
- Does the license affirmatively permit this training use?
- Does the service contract affirmatively permit this training use?
- Does permission cover downstream release under the intended terms?
- Are fallbacks disabled and the actual returned model/provider captured?
- Are copyright or text-and-data-mining rights reserved?

## Downstream obligations

- Attribution and notice:
- Naming requirements:
- Acceptable-use or field-of-use limits:
- Scale, revenue, or user thresholds:
- Patent, trademark, and retaliation clauses:
- Copyleft or downstream licensing:
- Territory restrictions:
- Retention, deletion, audit, or reporting duties:

## Data rights and safety

- Input/source provenance:
- Personal data status and re-identification assessment:
- Children's, biometric, medical, financial, or other sensitive data:
- Secrets, confidential material, leaked code, or access-controlled data:
- Large copyrighted works or output-similarity risk:
- Prohibited-use and dual-use assessment:
- Privacy/security/safety/regional reviews required and completed:

## Decision

- Decision (`approved`, `pending`, or `rejected`):
- Reason and unresolved conditions:
- Required re-review trigger:

Only an `approved` source is copied into the canonical registry. Translate the
review into every required `source-registry.json` field, then run:

```bash
cargo test --test unit docs_requirements_issue_834
```

