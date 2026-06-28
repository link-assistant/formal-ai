# Case study - Issue #481: telegraphic 3D-print ordering how-to

- **Issue:** [#481](https://github.com/link-assistant/formal-ai/issues/481)
- **Reported version:** 0.200.0 (WASM worker)
- **Reported prompt:** `how order 3d print in nan chang vietnam?`
- **Reported result:** `intent: unknown`
- **Pull request:** [#580](https://github.com/link-assistant/formal-ai/pull/580)
- **Raw data:** [`raw-data/`](./raw-data/) contains the captured issue JSON and comments.

## Requirement

The reported prompt is a procedural request with the infinitive connector omitted:
`how order ...` instead of `how to order ...`. The solver should reconstruct the
same procedural task (`order 3d print in nan chang vietnam`) and emit the normal
source-backed how-to discovery plan rather than falling to unknown.

The weak `how ...` form must stay narrow. Prompts such as `how glorp widgets?`
must not become procedural just because they start with `how`.

## Root Cause

`extract_procedural_how_to_task` only accepted slot-marked surfaces from the
`procedural_request` role, such as `how to ...`, `how can i ...`, and `what are
the steps to ...`. The reported prompt skipped that connector, so no procedural
surface matched and dispatch reached the unknown handler.

A broad `how ...` surface would be unsafe because `procedural_how_to` runs before
the more general `how_it_works` handler. It could steal arbitrary questions.

## Fix

Added two seed-backed roles:

- `procedural_request_elided_lead`: the weak slot-marked lead `how ...`
- `procedural_action_verb`: reviewed action verbs allowed after that weak lead

The Rust extractor and JS worker mirror now try the elided lead only after the
existing stronger how-to surfaces fail. They split the corrected task into
action/object and accept it only when the action is listed in the seed action
role. For this issue the reviewed action is `order`, so the reported prompt
formalizes as:

- request: `order 3d print in nan chang vietnam`
- action: `order`
- object: `3d print in nan chang vietnam`
- fallback query: `how to order 3d print in nan chang vietnam`

## Verification

- Added Rust regression coverage in
  `tests/unit/specification/reasoning_paths_procedures.rs`.
- Added a negative guard proving unknown bare actions after `how` do not route
  to `procedural_how_to`.
- Added `tests/e2e/tests/issue-481.spec.js` for the browser/WASM surface.
- Updated the procedural meta-recipe and role registry.
