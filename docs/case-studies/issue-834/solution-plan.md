# Issue #834 Solution Plan

| Requirement | Repository mechanism | Verification |
| --- | --- | --- |
| R834-01 | Keep the Unlicense unchanged; add explicit inbound-submission and third-party boundaries to the policy and contribution guide. | Assert the human-authored, authority, dedication, fallback, and third-party concepts. |
| R834-02 | Establish a single canonical training artifact directory, JSON approval registry, and reusable intake form. Start with the truthful empty state. | Parse the registry, validate every field in every entry, and compare its artifact paths with the recursive directory listing. |
| R834-03 | Add one contribution-surface prohibition covering issues, PRs, discussions, logs, fixtures, and commits, plus minimum-excerpt and removal rules. | Assert every prohibited source class and response action. |
| R834-04 | Separate copyright analysis from contracts; prohibit forbidden automation/circumvention and require written, scoped permission for an exception. | Assert the closed-provider examples and competing-model/terms rule. |
| R834-05 | Put model-specific conditions in policy but make the source form generic across families and routes. | Assert exact-version, attribution, naming, route, terms snapshot, Llama/Mistral/OpenRouter conclusions. |
| R834-06 | Adopt a no-real-personal-data training rule; treat pseudonymised and public data as still potentially personal. | Assert sensitive categories, synthetic/anonymity distinction, GDPR minimisation, and re-identification review. |
| R834-07 | Define excluded primary purposes, legitimate dual-use constraints, human approval, and incident containment. | Assert representative abuse categories and operational safeguards. |
| R834-08 | Record the project's present non-provider status and build a reclassification gate for the first model release. | Assert Article 53(a)-(d), systemic risk, enforcement date, and no current exemption claim. |
| R834-09 | Cross-reference the existing Unlicense while stating that mandatory law and safety work remain. | Assert **AS IS**, no warranty, non-waivable examples, and counsel boundary. |
| R834-10 | Preserve issue/PR/research evidence, add PR intake prompts, update the cumulative matrix and changelog, and run the complete test. | Whole-task traceability test plus local/CI checks. |

## Alternatives rejected

- **A prose-only legal FAQ.** It would not stop an undocumented artifact from
  entering a future pipeline.
- **Treat every public model as distillable.** Weight access, output training,
  provider contracts, inputs, and downstream distribution are separate facts.
- **Make OpenRouter's filter the authority.** The provider calls it
  best-effort and model/provider routing can change.
- **Register benchmark data as training data.** That would falsify the current
  architecture and create permission by relabeling.
- **Claim a broad EU open-source exemption now.** Formal AI has no trained GPAI
  model today, and the future exemption would remain limited.
