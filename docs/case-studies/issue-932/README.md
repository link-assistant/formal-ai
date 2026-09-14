# Issue 932 Case Study

Issue [#932](https://github.com/link-assistant/formal-ai/issues/932) asks for
the part of PR [#119](https://github.com/link-assistant/formal-ai/pull/119) that
was never delivered: a generated software project for a given language must be
built and run inside the `link-foundation/box` image that matches that language,
using the language's own traditional init commands, and that check must run in
CI. The implementation adds a data-declared language → image contract, two
harness scripts, a Docker-gated `cargo test`, and one CI matrix leg per language.

## 1. Collected evidence and timeline

The collector manifest under `raw-data/github/` records the source PR #119 with
all three of its comment channels, issue #932 and its (empty) comment stream,
and prepared PR #1009 with its three channels. PR #119 carries 18 conversation
comments, **zero** review comments and **zero** formal reviews, so the entire
requirement is stated in conversation. There are no screenshots or image
attachments anywhere in the source material.

- 2026-05-18T22:10:33Z: `konard` opened PR #119 ("Generalize software project
  request planning") against `main` from `issue-80-290e44192310`.
- 2026-05-19T04:44:57Z: first maintainer follow-up — formalize the message
  first, then meaning → reasoning → plan → code.
- 2026-05-19T06:06:22Z: the comment this issue quotes. In full, the relevant
  sentence is: "*also to reduce size of tests, we should prefer test each
  software project of a specific language inside such version of link foundation
  box docker image, that matches the language. And use traditional commands to
  init repositories for each language, so it is less work for us, and also for
  the user if he wants to do it manually.*"
- 2026-05-19T07:04:25Z: third follow-up asking for more examples and deeper
  reasoning tests.
- PR #119 was merged (`state: MERGED`, last updated 2026-05-19T07:39:14Z) with
  the planning generalization but with no box-image test leg: the repository has
  never had a CI job that runs a generated project inside a language image.
- 2026-08-04T13:49:24Z: issue #932 separated that unimplemented requirement into
  its own acceptance criteria.
- 2026-08-14T10:26:13Z: PR #1009 was prepared as a draft on branch
  `issue-932-ca968c816c06`.

## 2. Complete requirement matrix

The full matrix with per-row evidence is
`docs/requirements/issue-0932-box-language-projects.md` (R932-1 … R932-13). In
short: one CI leg per supported language (R932-1) that runs the traditional init
commands (R932-2) and proves the project builds and prints its output (R932-3),
gated like the other slow legs (R932-4), reachable from `cargo test` with a
graceful Docker skip (R932-5), manually verified first (R932-6), built from
English, Russian, Hindi and Chinese prompts (R932-7), pinned against a surveyed
tag list (R932-8), declared as data rather than YAML (R932-9), preserved as this
case study (R932-10), delivered in one PR with release metadata (R932-11), with
at least 20% of the smallest leaves authored through the real Agent CLI
(R932-12), plus the routing defect found while building the corpus (R932-13).

## 3. Reproduction and root cause

Asking the solver for the Go installation guide as a script — the exact prompt
the corpus generator issues — answered with a software-project *plan* instead of
a script. The reproducer is the unit test
`installation_conversion::go_module_install_guide_converts_to_a_script`; before
the fix it failed with `intent == "software_project_plan"`.

The root cause is not in the installation handler. `data/seed/handler-precedence.lino`
already ranks `installation_conversion` (42) ahead of `software_project` (46),
but `MethodRegistry::ordered_method_names_for_relevants` hoists *any* method
named by a `handler:<name>` relevant ahead of the whole precedence table. The
promotion gates lived in one hardcoded array in `src/intent_formalization.rs`,
and that array promoted `handler:software_project` and `handler:write_script`
while never emitting `handler:installation_conversion`. A guide that says
"create the project" and "build the project" therefore matched the promoted
software-project cues, and the declared precedence never got a chance to apply.

The fix adds the missing promotion in declared order, so a prompt that fires
both cues reaches the handler the seed already ranks first. The array moved to
`src/intent_formalization/prompt_relevants.rs` in the same change: the parent
module was at 897 of its reviewed 900-line ceiling
(`tests/unit/ci-cd/issue_999.rs`), and the split follows the earlier
`write_program_request` split rather than shrinking the explanation.

## 4. Existing components and image survey

`raw-data/box-image-tags.log` is the live Docker Hub survey and
`raw-data/box-images.log` the local pull sizes. Its conclusions are recorded as
data in `data/meta/box-image-survey.lino` — the Agent-CLI-authored leaf — so a
test, not a paragraph, is what stops the matrix naming an unpublished image.

- Six per-language variants exist under the `konard` namespace: `box-rust`,
  `box-python`, `box-js`, `box-go`, `box-java`, `box-ruby`. Every one publishes
  the same semver ladder up to `2.4.0` plus `latest`, with `-amd64`/`-arm64`
  suffixed manifests.
- `2.4.0` is the newest semver tag present on *all six* variants, so it is the
  tag the contract pins. `latest` was rejected: the check must not change
  meaning between two runs of the same commit.
- `konard/box-c`, `konard/box-cpp`, `konard/box-csharp` and `konard/box-dotnet`
  do not exist. C, C++ and C# are therefore recorded as
  `box_language_project_deferred` records naming the full `konard/box` image
  they are waiting for, instead of being silently dropped from the matrix.
- Local sizes: `box-rust` 7.81 GB, `box-java` 7.07 GB, `box-python` 6.86 GB,
  `box-go` 6.72 GB, `box-ruby` 6.58 GB, `box-js` 4.21 GB. One runner cannot hold
  several of these next to a cargo build, which is why the CI leg is a matrix —
  one image per runner — and why it frees runner disk space first.
- The containers run as user `box` (uid 1001) with `HOME=/home/box`, so the
  corpus is shipped in over a `tar` stream instead of a bind mount, which would
  otherwise land root-owned host files or mismatch the container uid.
- `box-js` installs Node through `nvm`, which a non-interactive `bash -c` does
  not source, so that record carries an explicit `shell_prelude`.

## 5. Implemented design

`data/meta/box-language-projects.lino` is the single contract: registry,
namespace, image tag, project directory, expected output, the four prompt
locales, and one `box_language_project` record per language carrying its image,
program file, code fence, prompt locale, optional shell prelude, whether the
init needs network access, and the ordered traditional init steps.
`src/box_language_projects.rs` reads it for Rust callers; both shell scripts
read the same file with `awk`, so the matrix, the tests and the harness cannot
drift apart.

`scripts/generate-box-language-corpus.sh` asks the running solver for the
program in all four locales and refuses to continue unless the four answers are
byte-identical, rebuilds the installation guide from the contract's init steps,
asks the solver to convert that guide into a shell script, and asserts every
declared command survives the conversion.
`scripts/verify-box-language-projects.sh` is the host half: it skips (exit 0)
when Docker is unreachable unless `STRICT=1`, then streams the generated corpus
into `docker run` and executes `scripts/run-box-language-project.sh` inside the
image, which runs the generated init script and fails unless the project
directory exists and the program prints `Hello, world!`. Legs whose init needs
no network run with `--network none`.

## 6. Requirement-by-requirement solution plan

1. Survey the published box images, then declare the language → image contract
   and its init commands as data through the real Agent CLI.
2. Add the Rust reader plus the unit invariants that keep contract, matrix and
   traditional commands aligned.
3. Build the generator/verifier/in-container harness and run it by hand for
   every language.
4. Expose the same check from `cargo test`, skipping gracefully without Docker.
5. Wire the CI matrix leg with the same gating, concurrency and timeout shape as
   the other slow legs, and list it in `pipeline-status`.
6. Fix the installation-guide routing defect the corpus exposed, and publish the
   case study, requirement matrix and release metadata.

## 7. Verification

`raw-data/` holds the evidence: `verify-<language>.log` for each of the seven
container runs, `repro-before.log` for the routing defect as it failed, and the
two image surveys. `self-hosting-authorship/` holds the real Agent-CLI session
log, the server log, the authored `box-image-survey.lino`, and the leaf
decomposition the 20% floor is measured against.

This is a CI and container change, not visual UI work, so before/after
screenshots and visual regression tests are not applicable. Final full-suite and
CI results are recorded on PR #1009 at the pushed SHA.
