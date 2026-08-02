# Candidate models for lawful local transformation or distillation review

Reviewed on 2026-08-01. This is a dated technical/legal shortlist, not an approval registry, legal opinion, global performance ranking, or authorization
to collect hosted output. “Top” here means ten distinct, recent, notable model
families whose first-party weight repositories expose a permissive license
signal. Newer is determined from the maintainer's release or repository record,
not a mutable hub popularity sort.

A permissive weight license can support local use, modification, fine-tuning,
and distribution of the licensed material. It does not automatically license
training data, third-party components, generated output, names or marks, or a
hosted API route. Distillation from generated output needs affirmative output
rights and must also satisfy hosted-service terms. Every candidate remains
`Pending` until the exact artifact and intended release pass
[`source-review.md`](source-review.md).

## Ten different model families

| # | Exact candidate | First-party license signal | Distillation boundary | Intake state |
| --- | --- | --- | --- | --- |
| 1 | [GLM-5.1](https://huggingface.co/zai-org/GLM-5.1) (GLM-5 family) | 2026 repository; MIT | Prefer local weights. Confirm the exact revision, notices, input provenance, output status, and any separate Z.ai API terms | Pending |
| 2 | [DeepSeek-V4-Pro](https://huggingface.co/deepseek-ai/DeepSeek-V4-Pro) (or V4-Flash, separately reviewed) | 2026 repository says code and model weights are MIT | The statement covers repository and weights, not an unrelated hosted route; preserve the MIT notice and review generated-output rights | Pending |
| 3 | [Granite 4.1 30B](https://huggingface.co/ibm-granite/granite-4.1-30b) | Released 2026-04-29; Apache-2.0 | Pin base versus instruct, preserve license/NOTICE obligations, review synthetic training provenance and output rights | Pending |
| 4 | [Qwen3.6 35B-A3B](https://huggingface.co/Qwen/Qwen3.6-35B-A3B) (newer Qwen3.5 line) | 2026 repository; Apache-2.0 | Verify the exact repository license at the pinned revision and do not transfer it to a differently licensed Qwen family member or provider route | Pending |
| 5 | [Mistral Small 4](https://mistral.ai/news/mistral-small-4/) | Released 2026-03-16 under Apache-2.0 | Acquire the specifically identified open weights; hosted Mistral service terms and output use require a separate review | Pending |
| 6 | [Apertus 70B 2509](https://huggingface.co/swiss-ai/Apertus-70B-2509) | Released 2025-09; Apache-2.0 with extensive first-party documentation | Keep the exact 2509 license, notices, acceptable-use material, and training-data documentation; do not copy the conclusion to a later gated variant | Pending |
| 7 | [OpenAI gpt-oss-120b](https://huggingface.co/openai/gpt-oss-120b) | Released 2025-08-05; Apache-2.0 weights | Local weight rights are separate from OpenAI hosted-output restrictions; preserve notices and review downstream output and safety obligations | Pending |
| 8 | [SmolLM3 3B](https://huggingface.co/HuggingFaceTB/SmolLM3-3B) | Released 2025; Apache-2.0 | Review the exact base/instruct artifact, its data mixture, notices, and generated-output status | Pending |
| 9 | [OLMo 2 0325 32B](https://huggingface.co/allenai/OLMo-2-0325-32B) | Released 2025-03; Apache-2.0 | Pin weights and code separately, preserve notices, and review Dolma/component rights for the proposed downstream use | Pending |
| 10 | [Phi-4-mini-instruct](https://huggingface.co/microsoft/Phi-4-mini-instruct) | Released 2025; MIT | Confirm the exact model card, notices, responsible-use material, provider route, and output rights; a hosted Azure route is a separate contract | Pending |

The list deliberately excludes candidates whose current artifact license is
non-commercial, research-only, use-case restricted, ambiguous, or known only
from an aggregator label. A model can be technically excellent and still be
incompatible with an unqualified public-domain release.

## Decision procedure

1. Pin repository, revision, file hashes, base/instruct variant, and provider
   route. Prefer locally acquired first-party weights over an aggregator.
2. Read the actual license file, model card, notices, acceptable-use policy, and
   provider contract. A hub tag or `distillable` filter is insufficient.
3. Determine whether the operation uses weights, outputs, hidden states,
   logits, synthetic traces, or a combination; record affirmative permission
   for each material.
4. Record attribution, patent, naming, trademark, field-of-use, scale, and
   downstream-license conditions.
5. Evaluate training-data provenance, privacy, safety, output similarity, and
   the intended release territory and license.
6. Disable unapproved fallback routes and capture the model/provider actually
   returned for every generation.
7. Add artifacts only after a named reviewer records `approved` in
   `data/training/source-registry.json`.

If a hosted contract prohibits use of output to develop a competing model, a
permissive license on some separately downloadable weights does not override
that prohibition. If the output terms are silent, the decision is not inferred
as permission; it remains pending or rejected.
