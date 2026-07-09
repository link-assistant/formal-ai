# Issue 483 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R483-1 | Provide an experimental small-model fallback for formalization. | `src/translation/model_fallback.rs`. |
| R483-2 | The model must choose from supplied formalization options instead of writing new formalization output. | `experimental_model_advice_can_only_select_existing_candidates` and `model_advisor_prompt_lists_bounded_formalization_options`. |
| R483-3 | The model fallback must be disabled by default. | `experimental_model_formalization_fallback_is_off_by_default`; browser default preference is `false`. |
| R483-4 | No model runtime or weights may load before explicit settings opt-in. | `tests/e2e/tests/issue-483.spec.js` checks initial script sources and the disabled selector. |
| R483-5 | The settings UI must display only hardware-fitting models. | `small_model_catalog_filters_hardware_and_sorts_by_public_rating` and the WebGPU/device-memory Playwright test. |
| R483-6 | Fitting models must be sorted by public rating. | Rust catalog sorting test and Playwright option-order assertion. |
| R483-7 | Downloads must be on demand only, with nothing bundled into the app package. | No dependency or artifact was added for WebLLM, Transformers.js, or model weights; the E2E test asserts no model script loads during default or settings-only states. |
| R483-8 | LLMs remain advisory and never control the formal system. | `select_formalization_candidate_with_model_advice` feeds accepted advice back into the normal selector; invalid advice is ignored. |
| R483-9 | Preserve research and raw evidence in a case-study folder. | This folder contains issue/PR snapshots, external metadata captures, verification logs, requirements, solution plans, research notes, and screenshot evidence. |

## Open Scope

The catalog and selector are intentionally small. Adding more models should add
captured public-rating metadata and a regression case for the relevant hardware
gate. Adding an actual browser inference runtime must use the same opt-in gate
and the same bounded advice API rather than letting model text become a
formalization source of truth.
