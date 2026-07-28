# Legal and Compliance Policy

Last reviewed: 2026-07-24

This is Formal AI's conservative operating policy for contributions, data
intake, model distillation, releases, and incident response. It is a project
control, not legal advice, and it does not replace advice from qualified counsel
for a particular model, dataset, provider contract, product, or jurisdiction.
When a right or obligation is uncertain, the project fails closed: do not ingest
the material until the uncertainty is resolved and recorded.

## Current state and scope

Formal AI is currently symbolic software. It does not ship trained neural model
weights and has no approved training or distillation sources. This means Formal
AI is not yet a provider of a trained general-purpose AI model. The empty,
machine-readable [`data/training/source-registry.json`](data/training/source-registry.json)
records that state; it is not permission to start an undocumented pipeline.

Every external item must be classified before it is added:

1. **Training or distillation artifact.** Model inputs, outputs, reasoning
   traces, preference data, fine-tuning data, weights, or other material that
   will update model parameters. These may exist only under
   `data/training/artifacts/` and need an approved registry entry.
2. **Benchmark or evaluation fixture.** Material used only to measure behavior,
   not to update parameters. Its provenance belongs in
   [`data/benchmarks/LICENSES.md`](data/benchmarks/LICENSES.md). The compact
   Nemotron samples are in this category and are not a training source.
3. **Runtime retrieval or cache.** Information fetched to answer a request
   without changing model parameters. Its sources are governed by
   `data/seed/sources-registry.lino`, the source's license and access terms, and
   privacy law.
4. **Community report or technical example.** Issue, pull-request, log, or
   prompt/response material used to reproduce a defect. Public visibility does
   not make it freely reusable and does not make it training data.

Changing a material's purpose is a new intake. A benchmark, runtime cache, or
GitHub example cannot silently become training data.

## Public-domain dedication and contribution rights

The repository's inbound and outbound terms are the
[`LICENSE`](LICENSE), currently the Unlicense. To the extent applicable law
allows, authors dedicate their copyright interest to the public domain. Where a
public-domain dedication is not effective, the Unlicense supplies permissive
fallback terms. Human-authored code, prompts, curation, selection, arrangement,
documentation, and other expressive contributions are included when their
authors intentionally submit them under those terms.

That dedication reaches only rights a contributor owns or is authorized to
license. It does not erase a third-party license, contract, database right,
trade secret, privacy or publicity right, moral right, patent, or trademark.
Third-party material keeps its own terms and must be identified. A contributor
must not represent model output or someone else's work as their own. If a
patent grant, trademark permission, or jurisdiction-specific waiver is needed,
obtain it separately before release.

The U.S. Copyright Office currently distinguishes purely AI-generated material
from human authorship and may protect human selection, arrangement, or
modification. That is not a global rule and not a blanket clearance for model
output: output can reproduce protected expression, reflect unlawful input, or
be restricted by contract. The project therefore reviews provenance and terms
even when copyright may not subsist in the raw output.

## Fail-closed source intake

Price is not a license. Labels such as "open", "free", "open weights", or
"distillable" are not approval. Before one byte is used for training or
distillation, a reviewer must complete
[`docs/legal/source-review.md`](docs/legal/source-review.md) and record all of
the following in the source registry:

- exact model or dataset name, upstream version/revision, provider and provider
  route;
- acquisition date and method, immutable hashes, artifact paths, and evidence
  paths;
- upstream license URL, applicable API or hosting terms URL, and the date or
  preserved terms snapshot that was checked;
- an affirmative basis for using outputs for training and for distributing the
  intended result—not an inference from silence;
- input-data provenance where supplied by the operator;
- attribution, notice, acceptable-use, naming, field-of-use, revenue/scale,
  territory, patent, and downstream-license conditions;
- copyright/TDM rights reservations, personal-data status, and any deletion or
  retention duty; and
- a named reviewer and `approved` decision.

CI compares every file in `data/training/artifacts/` with the approved registry.
An unregistered artifact, a missing required field, or a non-approved entry
fails the build. Re-review is required when a model version, route, provider,
terms, intended use, output license, territory, or release plan changes. If
permission cannot be verified, quarantine the payload outside the repository
and do not train on it.

No external training source is grandfathered. Community issue content and
historical benchmark data must pass the same review if their purpose changes.

## Copyright, research excerpts, and takedowns

Fair use in the United States and research, quotation, or text-and-data-mining
exceptions elsewhere are fact-specific defenses, not automatic permissions.
There is no safe number of words or percentage. The EU text-and-data-mining
rule for general uses additionally depends on lawful access and an absence of
an effective rights reservation.

Issues, pull requests, discussions, logs, fixtures, and documentation must not
contain:

- leaked or proprietary source code;
- paid, licensed, or access-controlled datasets without redistribution rights;
- large verbatim copyrighted works or bulk model output;
- credentials, private keys, trade secrets, or confidential material; or
- real personal data.

Use the minimum excerpt necessary for reproducible analysis, link to the
authorized source, identify its author/license/provider, and prefer a
description or hash when the payload is not needed. A short excerpt still
requires a case-specific basis; adding it to an issue does not approve it for
training.

On a credible rights, privacy, or terms complaint, maintainers stop downstream
use, quarantine the affected artifact, preserve non-sensitive provenance,
investigate registry dependants, and remove or replace material when warranted.
Security-sensitive reports should use GitHub's private security-advisory
channel rather than a public issue. A takedown does not decide the underlying
legal dispute and does not prevent compliance with a lawful preservation duty.

## Models, hosted services, and distillation

### Closed APIs

Do not use automated scraping, account rotation, rate-limit circumvention, or a
closed API to collect output for a competing model when the applicable contract
or Terms of Service prohibits it. Current public terms from OpenAI, Anthropic,
and the Gemini API include restrictions relevant to competing-model training or
service extraction. Copyright uncertainty does not cancel a contract
restriction.

An enterprise or research agreement can change the answer only when written
permission covers the exact account, model, provider route, volume, training
purpose, output distribution, and territory. Record that evidence without
committing confidential terms. Ordinary interactive use for debugging is not
automatically permission to retain the result as training data.

### Llama 3.3

Review the exact release; "Llama" is not one license. For
Llama 3.3, Meta's model card describes synthetic-data generation and
distillation as intended uses, but the Llama 3.3 Community License and
Acceptable Use Policy still control. A distributed product containing Llama
materials must display **Built with Llama** and carry the required notice. If
Llama 3.3 materials or outputs are used to train a distributed AI model, the
license's naming condition requires that model's name begin with **Llama**.
The license also has an additional-license condition for a licensee whose
products or services exceeded 700 million monthly active users in the preceding
month at the Llama 3.3 release date. Acceptable-use, attribution, patent, and
redistribution terms must also be reviewed.

Those conditions may be incompatible with a desired unqualified public-domain
release. Resolve and record the release name, notice placement, incorporation
of Llama materials, and distribution plan before approval.

### Mistral 7B and other open-weight models

Mistral 7B v0.1's official model card identifies its weights as Apache 2.0.
That is useful evidence for that exact model version, not a promise about every
Mistral model or hosted endpoint. Mistral publishes models under several
different licenses, and service terms can add restrictions. Apache 2.0 notice,
attribution, patent, modification, input-rights, provider-route, and intended
output-use questions still need review.

Apply the same exact-artifact rule to Qwen, Gemma, and every other family:
record the immutable model version, weight license, model card, acceptable-use
policy, hosting terms, and actual provider route. Never copy a conclusion from
another family member.

### OpenRouter and aggregators

OpenRouter documents `enforce_distillable_text` as a best-effort filter and
instructs users to verify the selected model's license. It is a useful routing
control, not a legal safe harbor and not sufficient approval. Aggregator labels
can lag, terms can change, and fallback routing can select a different model or
provider.

An approved pipeline must pin or capture the exact returned model and provider
route, enable the strongest applicable filter, independently verify the model
creator's license and the provider/aggregator Terms of Service, disable
unapproved fallbacks, and retain dated evidence. Any mismatch fails closed.

## Privacy and data governance

No real personal data enters training or distillation data. This includes
names tied to individuals, email or network identifiers, medical or financial
records, government identifier numbers, precise location, private
communications, voiceprints, biometric templates, faces, and images or text
that can identify a person. Do not collect children's data.

Pseudonymisation is not anonymisation: replacing a name with an identifier can
leave the person identifiable. Only purpose-built synthetic data or data
demonstrated to be irreversibly anonymous may be considered, and it still needs
documented re-identification testing and source review. Publicly accessible
personal data is not exempt.

For any processing outside training, apply GDPR principles where relevant:
lawfulness, fairness and transparency; purpose limitation; data minimisation;
accuracy; storage limitation; integrity and confidentiality; and
accountability. Establish a lawful basis, notices, retention/deletion handling,
data-subject rights, processor/controller roles, transfer mechanism, security
controls, and a data-protection impact assessment where required. Other
jurisdictions may impose additional biometric, child-data, health-data,
consumer, or deletion obligations.

## Prohibited uses and safety controls

Formal AI is general-purpose research software. The project does not accept or
distribute code, data, tuned behavior, examples, or integrations whose primary
purpose is:

- creating, sexualizing, grooming, locating, or facilitating CSAM or child
  exploitation;
- deploying malware, credential theft, destructive intrusion, or instructions
  to bypass security without authorization;
- developing biological or chemical weapons, or enabling credible physical
  harm;
- unlawful surveillance, biometric identification, discrimination, trafficking,
  or targeted harassment; or
- evading legal, platform, or safety controls.

Legitimate defensive-security, safety, and scientific work must be scoped to an
authorized target, use least privilege and isolated fixtures, exclude harmful
payloads where a safe substitute works, document the benefit and residual risk,
and require human review before consequential action. Releases must retain
applicable safeguards, provenance, reporting paths, and operator controls.

Maintainers triage safety reports, disable or quarantine an unsafe distribution
path when necessary, document an incident decision, notify affected parties
where legally required, and add a regression control. This policy cannot
guarantee that general-purpose software will never be misused.

## EU AI Act assessment

As of this review, the repository has no trained neural weights and is not yet a
provider of a general-purpose AI model, so no free and open-source exemption is
currently being claimed. Reassess before training, substantially modifying, or
placing a model on the EU market under the Formal AI name.

For a future general-purpose model, Regulation (EU) 2024/1689 Article 53 does
not create a complete open-source exemption:

- a model released under a qualifying free and open-source licence with public
  parameters may be exempt from Article 53(1)(a) technical documentation and
  (b) integrator information;
- the exemption does not remove Article 53(1)(c), the copyright policy to
  comply with EU copyright law and rights reservations, or (d), the public
  training-content summary;
- it does not apply to a general-purpose model with systemic risk; and
- copyright, data protection, prohibited-practice, product, transparency, and
  other applicable duties remain.

Do not treat "open weights", lack of price, lack of current monetisation, or a
public repository as sufficient. Confirm the license, public parameters,
provider role, compute/capability classification, systemic-risk status,
placing-on-market facts, and any downstream substantial modification.

General-purpose-model obligations began applying on 2 August 2025. European
Commission enforcement for new models begins on 2 August 2026, while qualifying
models placed on the market before 2 August 2025 have a later compliance
timeline. These dates and Commission guidance may change; counsel must verify
the current text at each release.

## Other jurisdictions and legal domains

Approval for one territory does not imply worldwide approval. Before a public
training run or model release, identify intended territories and review at
least:

- copyright and text-and-data-mining rules, including lawful access, opt-outs,
  database rights, and output similarity;
- contract, anti-circumvention, trade-secret, patent, trademark, publicity, and
  moral rights;
- privacy, biometrics, children, health, employment, credit, education, and
  consumer-protection law;
- product safety, professional-services, accessibility, discrimination,
  election, content, and sector-specific rules; and
- export controls, sanctions, cloud/model controls, and local registration or
  transparency duties.

This list is a review trigger, not a representation that every regional nuance
has been resolved.

## Warranty, liability, and mandatory law

The Unlicense states that the software is provided **"AS IS"**, with no warranty
and a limitation of liability. Contributors and distributors must preserve the
applicable notice.

That language does not replace compliance work and is not universally
enforceable. It cannot be assumed to waive liability that mandatory law does
not permit parties to waive, including rules that may apply to intentional harm,
gross misconduct, mandatory consumer protection, privacy/data protection,
product safety, or direct physical harm. It also gives no warranty that a model,
dataset, output, or third-party service is accurate, non-infringing, safe, or
fit for a regulated purpose.

Do not market the disclaimer as a safeguard. Use testing, human oversight,
security controls, incident response, insurance where appropriate, and
jurisdiction-specific counsel.

## Review and evidence

The legal/compliance owner for a proposed source or release must:

1. complete the source review and registry entry;
2. obtain privacy, safety, security, and regional review where triggered;
3. re-check linked primary sources and terms immediately before acquisition and
   release;
4. record approvals, rejected alternatives, hashes, and material changes; and
5. reopen this audit at least annually or after a material legal, provider,
   model, data-purpose, or distribution change.

Community review is welcome, but silence, a label, or a merged pull request is
not legal approval.

## Primary references

- U.S. Copyright Office, [Copyright and Artificial Intelligence](https://www.copyright.gov/ai/)
  and [Fair Use Index](https://www.copyright.gov/fair-use/)
- Directive (EU) 2019/790,
  [Article 4 text-and-data mining](https://eur-lex.europa.eu/eli/dir/2019/790/oj)
- Regulation (EU) 2016/679,
  [GDPR](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679)
- Regulation (EU) 2024/1689,
  [EU AI Act](https://eur-lex.europa.eu/eli/reg/2024/1689/oj)
- European Commission,
  [general-purpose AI model guidance](https://digital-strategy.ec.europa.eu/en/policies/guidelines-gpai-providers)
- Meta, [Llama 3.3 model card](https://github.com/meta-llama/llama-models/blob/main/models/llama3_3/MODEL_CARD.md)
  and [license](https://github.com/meta-llama/llama-models/blob/main/models/llama3_3/LICENSE)
- Mistral AI, [Mistral 7B model card](https://docs.mistral.ai/models/model-cards/mistral-7b-0-1)
- OpenRouter, [distillation guidance](https://openrouter.ai/docs/cookbook/evaluate-and-optimize/distillation)
  and [Terms of Service](https://openrouter.ai/terms)
- OpenAI, [Services Agreement](https://openai.com/policies/services-agreement/)
- Anthropic, [Commercial Terms](https://www.anthropic.com/legal/commercial-terms)
- Google, [Gemini API Additional Terms](https://ai.google.dev/gemini-api/terms)
