# Issue 12 Case Study: Holistic Requirements and Vision

## Summary

Issue [#12](https://github.com/link-assistant/formal-ai/issues/12) asks for a holistic synthesis of requirements from issue #1, issue #4, issue #6, issue #8, issue #10, and the related pull request comments. It also asks for root vision documents: [`../../../VISION.md`](../../../VISION.md), [`../../../GOALS.md`](../../../GOALS.md), and [`../../../NON-GOALS.md`](../../../NON-GOALS.md).

The core product direction is a symbolic assistant whose live state is an associative operational space. The assistant should prefer small seed knowledge plus dynamic construction of a transparent link network over a large memoized answer database. The network should store meanings, source events, traces, commands, permissions, handlers, and answer evidence in Links Notation and a Links Data Store backed by doublet links.

## Collected Data

Fresh GitHub evidence is stored in `raw-data/`:

- `issue-1.json`, `issue-4.json`, `issue-6.json`, `issue-8.json`, `issue-10.json`, `issue-12.json`: issue metadata, bodies, labels, timestamps, and embedded comments returned by `gh issue view`.
- `issue-*-comments.json`: issue conversation comments fetched through the GitHub issue comments API.
- `pr-2.json`, `pr-5.json`, `pr-7.json`, `pr-9.json`, `pr-11.json`, `pr-13.json`: related pull request metadata.
- `pr-*-conversation-comments.json`: general pull request discussion comments.
- `pr-*-review-comments.json`: inline pull request review comments.
- `pr-*-reviews.json`: pull request review records.
- `recent-merged-prs.json`: recent merged PRs used to match repository documentation style.

The most important pull request comments for this synthesis are:

- PR [#2](https://github.com/link-assistant/formal-ai/pull/2): requested a richer markdown chat UI, markdown input behavior, demo dialogs that start with `Hi` or `Hello`, randomized language prompts, and a demo toggle.
- PR [#7](https://github.com/link-assistant/formal-ai/pull/7): carried the issue #10 UI feedback into the prior UI PR, including Preview removal, issue reporting, and identity answers.
- PR [#9](https://github.com/link-assistant/formal-ai/pull/9): requested Telegram polling as the default CLI mode, keeping webhook as an option, publishing the Telegram CLI through the Cargo crate, and configuring CLI flags through `lino-arguments`.

No inline review comments or formal reviews were present in the related PR raw data at collection time.

## Prior Case Studies

Existing case-study files were used as already reviewed repository evidence:

- [`../issue-1/README.md`](../issue-1/README.md): formal AI proof of concept, OpenAI-shaped APIs, symbolic rules, Links Notation data, web demo, and dataset boundaries.
- [`../issue-4/README.md`](../issue-4/README.md): GitHub Pages deploy root-cause analysis and CI/CD regression coverage.
- [`../issue-6/README.md`](../issue-6/README.md): demo-mode default behavior, countdown feedback, and diagnostics gating.
- [`../issue-8/README.md`](../issue-8/README.md): Telegram interface, code execution metadata, polling CLI, and `lino-arguments`.
- [`../issue-10/README.md`](../issue-10/README.md): issue-reporting links, identity intent, and preview removal.

## Online Research

External sources checked for current facts and component fit:

- [Deep.Foundation](https://deep.foundation/) describes link-based associative events, associative packages, handler code in associative memory, Docker-backed execution, and semantic permissions.
- [deep-foundation on GitHub](https://github.com/deep-foundation) provides implementation references for Deep.Foundation packages and clients.
- [link-foundation/link-cli](https://github.com/link-foundation/link-cli) is a CLI for manipulating links with a single substitution operation, `.lino` export, and persistent transformation triggers.
- [link-foundation/links-notation](https://github.com/link-foundation/links-notation) describes Links Notation as a way to convert strings containing references and links into link lists and back.
- [link-foundation/lino-objects-codec](https://github.com/link-foundation/lino-objects-codec) provides universal object serialization into Links Notation, including Rust support and human-readable indented output.
- [linksplatform/doublets-rs](https://github.com/linksplatform/doublets-rs) is a Rust implementation of doublet associative storage links with index, source, and target fields.
- [konard/problem-solving](https://github.com/konard/problem-solving) outlines a problem-solving loop based on formulation, failing tests, implementation, debugging, refactoring, and documentation.
- [Wikidata](https://www.wikidata.org/wiki/Wikidata:Main_Page) is a free open knowledge base editable by humans and machines and exports structured data in standard formats.
- [Wikifunctions](https://www.wikifunctions.org/wiki/Wikifunctions:Main_Page) is a collaborative Wikimedia library of functions across natural and programming languages.
- [Rosetta Code](https://rosettacode.org/wiki/Rosetta_Code) is a programming chrestomathy with tasks implemented across many languages.
- [WebVM](https://github.com/leaningtech/webvm) runs a client-side Linux virtual environment in HTML5/WebAssembly and can run unmodified Debian with development toolchains.
- [CheerpX](https://cheerpx.io/) provides the browser x86 virtualization technology behind WebVM.
- [link-assistant/agent](https://github.com/link-assistant/agent) is an OpenCode-compatible agent CLI intended for isolated unrestricted environments.

## Requirements From Source Threads

Issue #1 establishes the baseline: a formal or symbolic AI, no GPU-required neural inference, OpenAI-compatible Chat Completions and Responses APIs, Links Notation data, repository datasets, library/CLI/API/Docker/GitHub Pages surfaces, formal reasoning direction, and issue case studies with raw data.

PR #2 expands the web demo requirements: markdown messages, markdown input behavior, randomized demo dialogs that start with a user greeting, and randomized hello-world language prompts.

Issue #4 adds operational discipline: preserve CI/CD logs, reconstruct timelines, identify real root causes, compare templates, report upstream template defects, and add regression coverage.

Issue #6 makes the web demo user-facing by default: demo mode starts on, the countdown updates every second, diagnostics stay off by default, and normal messages remain free of diagnostic distraction.

Issue #8 adds execution-aware assistant behavior: Telegram private and public chats, generated code should be compiled or run when possible, output and limitations must be reported, long reasoning failures need logs, and every interface must know its execution limits.

PR #9 adds the second Telegram iteration: the Telegram CLI should run polling by default, webhook should remain optional, the command should ship in the Cargo crate, and CLI configuration should use `lino-arguments`.

Issue #10 adds feedback and identity: remove the Preview button, generate prefilled GitHub issue links for unknown prompts, allow issue reporting on any dialog, answer "Who are you?" variations, and preserve case-study evidence.

Issue #12 adds the holistic architecture: less preexisting database content, more dynamic associative construction, the associative network as the AI itself, add-only history, user-queryable transparency, doublet links instead of Deep.Foundation triplets, a dynamic type system, cached source access, associative packages, handlers, permissions, substitution triggers, universal problem solving, network visualization, chat-first interaction, agent mode with isolated execution, multilingual conversation, and translation through Links Notation as a language of meaning.

## Holistic Requirements

| ID | Requirement | Source | Status in this PR |
| --- | --- | --- | --- |
| H1 | Preserve raw issue and PR evidence for the holistic analysis under `docs/case-studies/issue-12`. | Issue #12 | Implemented in `raw-data/`. |
| H2 | Produce root `VISION.md`, `GOALS.md`, and `NON-GOALS.md`. | Issue #12 | Implemented. |
| H3 | Prefer the smallest useful seed dataset plus dynamic knowledge construction. | Issue #12, issue #1 | Documented as a guiding goal. |
| H4 | Treat the associative network as the assistant's live state. | Issue #12 | Documented in `VISION.md`. |
| H5 | Use Links Data Store and doublet links as the long-term storage direction. | Issue #12 | Documented as architecture direction. |
| H6 | Represent the dynamic type system as `Type -> SubType`, `SubType -> SubType`, and `SubType -> Value`. | Issue #12 | Documented in `VISION.md`. |
| H7 | Preserve add-only history and derive current state from logged events. | Issue #12 | Documented in vision, goals, and non-goals. |
| H8 | Make every answer, step, command, and decision traceable. | Issue #12, issue #4, issue #8 | Documented as transparent reasoning. |
| H9 | Keep chat as the default interface and expose the link graph only when useful. | Issue #12, issue #6 | Documented. |
| H10 | Keep chat autonomy bounded to the current user message. | Issue #12 | Documented in `VISION.md` and `NON-GOALS.md`. |
| H11 | Provide explicit agent mode with visible actions and isolated execution. | Issue #12, issue #8, link-assistant/agent | Documented as a product goal. |
| H12 | Support code generation in popular languages with compile/run evidence where possible. | Issue #1, issue #8 | Already partially implemented; documented as near-term focus. |
| H13 | Report unsupported execution honestly across CLI, API, web, and Telegram. | Issue #8 | Already partially implemented; preserved as a product goal. |
| H14 | Use associative packages, handlers, permissions, and triggers as long-term computation mechanisms. | Issue #12, Deep.Foundation, link-cli | Documented as architecture direction. |
| H15 | Search external sources when local links are insufficient and cache source accesses with provenance. | Issue #12 | Documented as reasoning goal. |
| H16 | Translate between natural languages, programming languages, and Links Notation as a language of meaning. | Issue #12 | Documented. |
| H17 | Split overloaded names into distinct meanings to reduce contradictions. | Issue #12 | Documented in `VISION.md`. |
| H18 | Preserve existing requirements from issue #1, issue #4, issue #6, issue #8, issue #10, and PR comments. | Issue #12 | Added to this case study and `docs/REQUIREMENTS.md`. |
| H19 | Keep the documentation set testable so future deletions fail CI. | Issue-solver workflow | Implemented in `tests/unit/docs_requirements.rs`. |

## Root Cause

Before this PR, the repository had strong per-issue case studies but no single document that connected them into a product vision. `docs/REQUIREMENTS.md` listed implementation requirements through issue #8, and the issue #10 case study described UI feedback work, but there were no root-level vision, goals, or non-goals documents. There was also no fresh issue-12 evidence bundle.

The gap was therefore not missing runtime behavior. The gap was missing traceable synthesis.

## Design Decisions

Root documents live at the repository root because they define project-wide direction rather than a single issue implementation detail. The issue-12 case study remains under `docs/case-studies/issue-12` to preserve source evidence, requirement extraction, external research, and solution planning.

The new test checks for the presence of the project-wide documents and key concepts. This is intentionally a documentation completeness test, not a markdown style checker. It catches accidental deletion of the issue-12 deliverables without making prose hard to edit.

The architecture language stays explicit about current versus future state. The current repository is still a deterministic proof of concept. The vision documents describe the target direction and near-term path without claiming that the link-store-backed reasoning loop already exists.

## Solution Plan

Evidence and synthesis:

- Store fresh raw GitHub issue and PR evidence in `docs/case-studies/issue-12/raw-data`.
- Reuse reviewed case studies for issues 1, 4, 6, 8, and 10 as local evidence.
- Document PR-comment requirements that were not fully visible from issue bodies alone.
- Cite current external sources for Deep.Foundation, Link Foundation components, doublets, problem-solving, datasets, browser execution, and agent mode.

Vision:

- Define the assistant as a live associative network.
- Prefer small seeds and on-demand knowledge construction.
- Use doublet links, Links Notation, and Links Data Store as the long-term substrate.
- Treat add-only history and transparent reasoning as core requirements.
- Separate bounded chat mode from explicit isolated agent mode.

Goals:

- Convert the vision into architecture, product, reasoning, documentation, and near-term goals.
- Keep implemented surfaces tied to the same symbolic core.
- Focus next work on trace links, link-store-backed reasoning, network visualization, and execution-aware code generation.

Non-goals:

- Exclude GPU-required neural inference, memoized answer caches, hidden autonomy, untracked external context, destructive memory updates by default, and false claims about execution.
- Clarify that the current web demo, Telegram bot, and desktop path have bounded roles.

Regression coverage:

- Add `docs_requirements::issue_12_vision_documents_are_present_and_traceable`.
- The test fails before the documents exist and passes after the issue-12 documentation set is present.

## Known Boundaries

- This PR does not implement the full link-store-backed associative reasoning loop.
- This PR does not import Wikidata, Wikipedia, Wikifunctions, Rosetta Code, or Deep.Foundation packages.
- This PR does not add graph visualization, handler execution, permission logic, or persistent link triggers.
- This PR does not change runtime answers, API behavior, Telegram behavior, or the web demo.
- Future implementation PRs should convert these documented requirements into smaller testable runtime milestones.

## Verification

The reproducing test failed before documentation was added because `VISION.md` did not exist. After the documentation update, the expected checks are:

```text
cargo test --test unit docs_requirements::issue_12_vision_documents_are_present_and_traceable
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-features --verbose
cargo test --doc --verbose
rust-script scripts/check-file-size.rs
rust-script scripts/check-changelog-fragment.rs
git diff --check
npm --prefix tests/e2e run test:local
```
