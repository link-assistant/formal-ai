# Issue 932: exercise generated language projects inside matching box images

Source: [issue #932](https://github.com/link-assistant/formal-ai/issues/932) and
the maintainer comment on [PR #119](https://github.com/link-assistant/formal-ai/pull/119)
that asked to "test each software project of a specific language inside such
version of link foundation box docker image, that matches the language" and to
"use traditional commands to init repositories for each language".

| ID | Requirement | Status and evidence |
| --- | --- | --- |
| R932-1 | Give every language the project-generation handlers support a CI leg that pulls the matching `link-foundation/box` image variant. | Implemented by the `box-language-projects` matrix job in `.github/workflows/release.yml`, one leg per language in `data/meta/box-language-projects.lino`. |
| R932-2 | Run the language's traditional init/build commands (`cargo new`, `npm init`, `go mod init`, …) against a Formal-AI-generated project inside that container. | Implemented by `scripts/generate-box-language-corpus.sh` (project + init script from solver answers) and `scripts/run-box-language-project.sh` (runs them in the image). |
| R932-3 | Prove the project actually builds and runs, not merely that the container started. | `scripts/run-box-language-project.sh` fails unless the init script exits zero, the project directory exists, and the program prints `Hello, world!`. |
| R932-4 | Wire it as a CI job gated like the other slow legs. | `box-language-projects` uses the `detect-changes` gate, branch-cancelling concurrency, `timeout-minutes: 30`, and is listed in `pipeline-status`; asserted by `tests/unit/ci-cd/issue_932.rs`. |
| R932-5 | Offer the same check from `cargo test`, running it through `docker run` when Docker is available and skipping gracefully otherwise. | `tests/integration/issue_932_box_language_projects.rs` mirrors `scripts/verify-docker-runtime.sh`: it skips when the daemon is unreachable and, by default, runs only the images already present locally. |
| R932-6 | Verify manually against sample generated projects before trusting CI. | `docs/case-studies/issue-932/raw-data/verify-*.log` records the container runs for every language in the matrix. |
| R932-7 | Build the corpus from prompts in English, Russian, Hindi, and Chinese. | `box_language_prompt` records carry all four locales; the generator requires the four answers to be byte-identical, and `tests/unit/issue_932_box_language_projects.rs` asserts the same property. |
| R932-8 | Survey the published `link-foundation/box` image tags before pinning any. | `docs/case-studies/issue-932/raw-data/box-image-tags.log` is the live survey, recorded as data in `data/meta/box-image-survey.lino`; `every_matrix_image_was_actually_found_on_the_registry` fails if the matrix names an image the survey did not find, or drifts off the pinned `2.4.0`. Section 4 of the case study explains why C, C++, and C# stay deferred. |
| R932-9 | Keep the language → image mapping reviewable as data, not as workflow YAML. | `data/meta/box-language-projects.lino`, read by `src/box_language_projects.rs`, by both shell scripts, and by the CI-matrix invariant test. |
| R932-10 | Preserve the issue evidence, requirement matrix, root cause, and library survey as a case study. | `docs/case-studies/issue-932/`. |
| R932-11 | Deliver the issue in one PR with release metadata. | PR [#1009](https://github.com/link-assistant/formal-ai/pull/1009) and a minor changelog fragment. |
| R932-12 | Produce at least 20% of the smallest task leaves through the real Formal AI Agent CLI and preserve proof. | 1 of the 5 leaves in `docs/case-studies/issue-932/self-hosting-authorship/decomposition.lino` is Agent-CLI-authored: `experiments/issue_932_self_authoring/run.sh` drives the real CLI against `formal-ai serve` to write `data/meta/box-image-survey.lino`, and `tests/unit/issue_932_self_authoring.rs` asserts the committed file is byte-for-byte the captured artifact. |
| R932-13 | Keep an install guide that carries a traditional init command routed to `installation_conversion`. | Discovered while building the corpus: `go mod init` guides were answered by the software-project planner. Fixed by the missing promotion gate in `src/intent_formalization/prompt_relevants.rs`; regressions in `tests/unit/installation_conversion.rs` and `tests/unit/issue_932_box_language_projects.rs`. |
