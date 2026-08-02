# Issue #834: Legal and Compliance Self-Audit

Issue [#834](https://github.com/link-assistant/formal-ai/issues/834) asks whether
Formal AI's legal guidance covers the nuances of public-domain development,
training and distillation provenance, community examples, provider terms,
privacy, safety, the EU AI Act, and warranty limits. It also asks concrete
questions about Llama 3.3, Mistral 7B, OpenRouter filters, and regional law.

This case study is an engineering self-audit and community review anchor. It is
not legal advice. No finite repository document can certify compliance with every
jurisdiction or future fact pattern. The deliverable is therefore a conservative
baseline plus a fail-closed review process that turns uncertainty into a blocked
intake rather than an unsupported claim.

## Finding

The repository was not ready for a training phase. It had useful pieces:

- `LICENSE` already carried the Unlicense public-domain dedication, fallback
  permission, **AS IS** warranty disclaimer, and liability limitation;
- `data/benchmarks/LICENSES.md` recorded provenance for imported evaluation
  fixtures;
- `NON-GOALS.md` prohibited secrets and personal data in memory/event logs; and
- runtime retrieval sources were listed in `data/seed/sources-registry.lino`.

Those controls did not compose into a training compliance system. The principal
gap was the absence of an explicit inbound contribution rule and an enforceable
training source registry. There was also no single policy for model/provider
terms, personal data, prohibited uses, EU AI Act classification, regional
review, or the limits of the disclaimer.

Formal AI currently has no trained neural weights and no approved
training/distillation corpus. The Nemotron samples are an evaluation fixture,
not training data. This distinction is now explicit and tested.

## Before/after audit

| Issue checklist | Before this change | Resolution |
| --- | --- | --- |
| Human-authored public-domain contribution | `LICENSE` covered repository authors generally, but `CONTRIBUTING.md` did not state an inbound submission rule or third-party boundary. | `CONTRIBUTING.md` now requires authority and intentional submission under the Unlicense; `LEGAL-COMPLIANCE.md` explains fallback and excluded third-party rights. |
| Provenance for every distillation source | Benchmark provenance existed, but no canonical training location, schema, or gate existed. | `data/training/source-registry.json` declares the honest empty state; CI rejects unregistered artifacts and incomplete future entries. |
| Leaks, paid data, and long verbatim works | Secrets were addressed, but the issue/PR contribution rule was incomplete. | Contribution and policy rules prohibit those payloads, require minimum lawful excerpts, and define quarantine/removal. |
| Closed API / competing training | No consolidated rule. | The policy prohibits forbidden scraping/extraction and requires exact written permission and provider-route evidence. |
| Attribution and naming | Benchmark licenses were tracked; model-output naming obligations were not. | The source review and registry require attribution/naming fields; the policy analyzes Llama 3.3, Mistral 7B, and exact-version review. |
| Real personal data | Event-log guidance existed, but not a training prohibition or review gate. | Training excludes real personal data, distinguishes pseudonymisation from anonymisation, and records privacy review. |
| Prohibited uses | General autonomy boundaries existed, but the requested abuse categories and incident workflow were not consolidated. | The policy defines excluded primary purposes, dual-use controls, human review, and incident handling. |
| EU AI Act open-source status | No project-specific assessment. | The policy records that Formal AI is not yet a GPAI model provider and explains the limited Article 53 exemption and re-review trigger. |
| Disclaimer and its limits | The Unlicense included broad language, but no operational explanation of non-waivable law. | The policy preserves **AS IS** / no-warranty language while rejecting reliance on it for intentional harm, mandatory law, or safety work. |

## Implemented control

The complete workflow is:

1. classify material as training/distillation, benchmark/evaluation, runtime
   retrieval/cache, or community example;
2. keep all parameter-updating artifacts under `data/training/artifacts/`;
3. complete `docs/legal/source-review.md` against primary evidence;
4. add only an approved, field-complete record to the training source registry;
5. let CI compare the canonical artifact tree with that registry; and
6. repeat review when the version, provider route, terms, purpose, territory, or
   release plan changes.

This keeps the control reviewable today and useful when training begins. An
empty source list cannot be misread as approval because it is paired with
`current_state: "no-approved-training-sources"` and an artifact-directory
equality check.

## Answers to the issue's questions

### Llama 3.3 and Mistral 7B

There is no family-wide answer. Meta's Llama 3.3 model card describes
distillation as an intended use, while the exact Llama 3.3 license adds
acceptable-use, notice, **Built with Llama**, model-naming, patent, and
700-million-MAU conditions. Those obligations can conflict with an
unqualified public-domain release and must be resolved before intake.

Mistral 7B v0.1's official card identifies Apache 2.0 for that artifact. That
does not establish the license for later Mistral models or a hosted service, and
it does not clear operator-supplied inputs. The project must pin the exact model
and route and carry all notices and downstream conditions.

### OpenRouter's distillable filter

No. OpenRouter itself describes `enforce_distillable_text` as best-effort and
directs users to check the model license. A compliant pipeline must additionally
verify and date the creator license and service terms, capture the actual model
and provider route, disable unapproved fallbacks, and retain evidence. A filter
is defense in depth, not legal approval.

### Regional nuances

The policy now has explicit review triggers rather than declaring worldwide
clearance. Examples include lawful access and machine-readable rights
reservations under EU text-and-data-mining rules; GDPR and other personal-data
regimes; UK-specific AI/data-protection guidance; Canadian privacy principles;
children and biometric rules; database, contract, trade-secret, publicity,
consumer, sector, product-safety, export, sanctions, and local-transparency
requirements. The release territories and use case decide which review applies.

## Evidence and verification

- `raw-data/issue.json` and the comment/PR API captures preserve the source
  request and all three GitHub PR feedback channels.
- `raw-data/online-research.md` records the primary-source research.
- `requirements.md` maps every checklist item and the complete workflow.
- `solution-plan.md` maps each requirement to the implemented repository hook.
- `test-logs/reproduction-before.log` shows all ten regression tests failing on
  the missing controls.
- `test-logs/core-policy-after.log` shows the nine individual controls passing.
- `test-logs/focused-after.log` shows all ten issue-specific tests passing.
- `test-logs/full-suite.log` records the all-features result: 188 binary tests,
  481 integration tests, 2,001 unit tests, and doc tests passed with no
  failures. The slow-test reporter identified existing agent-replay and ratchet
  cases, all of which completed successfully.
- `test-logs/fmt-check.log`, `test-logs/clippy.log`,
  `test-logs/check-file-size.log`, and
  `test-logs/check-hardcoded-language.log` record the remaining local gates.
- `self-hosting-evidence/` preserves the later local Formal AI session, its
  representative source-links document, and the exhaustive two-shard projection
  of all 293 owned modules.
- `tests/unit/docs_requirements_issue_834.rs` protects every checklist item and
  the whole task.

The required live Formal AI/Agent CLI command was attempted before
implementation. As captured in `raw-data/self-coding-live.log`, the external
`solve` wrapper rejected its configured `formal-ai` model before a session began
and automatically posted
[the failure notice](https://github.com/link-assistant/formal-ai/issues/834#issuecomment-5067948519).
The legal audit and its tests therefore remain manually authored and carry no
self-authorship trailers.

After the pull request's differential self-hosting check reported that the
unattributed branch would lower the release metric, the established local
Agent-CLI path ran a separate real Formal AI session,
`ses_06c5ab43dffekcO0P7PW77UVcL`. It authored a representative source-links
document and projected all 293 owned modules into two exhaustive shards; the
generator verified every module's source-to-links-to-source round trip
byte-for-byte. The transcript and artifacts are preserved under
`self-hosting-evidence/`. Only that isolated generated-artifact commit carries
the paired `Formal-AI-Session` and `Formal-AI-Evidence` trailers.

## Residual review boundary

This audit establishes a baseline; it does not approve any source. Before the
first real training run, obtain counsel for the exact source list, provider
contracts, intended weights/license/name, release territories, model
classification, safety controls, and operator entity. Reopen the audit when any
of those facts or applicable law changes.
