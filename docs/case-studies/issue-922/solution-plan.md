# Issue 922 Solution Plan

## Real Experience And Reproduction

- Execute the checked-in recursive recipe for distinct solved-problem shapes;
  do not substitute synthetic `ExecutionTrace` fixtures for acceptance proof.
- Add the smallest regression that demonstrates the missing EventLog-to-method
  bridge before implementing it.
- Normalize only stable control-flow kinds so prompt payloads and the registry's
  own serialized bytes cannot contaminate a reusable method identity.

Existing components: `recipe_interpreter`, `EventLog`, and issue #531's
`algorithm_discovery` support/held-out split.

## Proposal And Trust Boundaries

- Project every discovered candidate into reviewable method data, retaining
  support IDs, held-out IDs, and exact failure reasons.
- Admit only held-out-validated candidates to `PromotionProposal`.
- Leave gates empty at the learner boundary so issue #656's allow-listed replay
  remains authoritative for commands, floors, observations, and digests.
- Keep proposal parsing side-effect-free and reject incomplete or duplicate
  adopted seed entries.

Existing components: `promotion`, its append-only decision events, seed parser,
and `MethodRegistry`.

## Human-Confirmed Adoption

- Run the reviewed proposal through `formal-ai improve --promote --apply
  --confirm` in a clean local fixture repository.
- Require all canonical regression floors, retain the local review branch, and
  compare the materialized bytes to the production seed.
- Load the promoted record as registry-visible link data without manufacturing
  a compiled method handler.

Existing components: the issue #656 materializer and issue #559 registry.

## Verification And Traceability

- Exercise rejected promotion and assert its reason/evidence remains durable.
- Run recursive recipe/source parity after adoption and then all contributor
  checks.
- Replay the final bytes through a real external Agent CLI and retain raw
  machine-readable evidence.
- Synchronize requirements, meta-algorithm design, roadmap, case study, and the
  release-triggering changelog fragment in PR #1005.
