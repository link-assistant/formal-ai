---
bump: minor
---

### Added
- Issue #1085 (D2.3): the self-authored loop is a composite action, `.github/actions/author-with-formal-ai`, that another repository can install. Formal AI comes from the published container by default so a consuming repository spends no compile, the task contract can be relaxed to attempt every new issue rather than only labelled ones, and the shared scripts are fetched from this repository rather than vendored. `self-authored-pull-request.yml` is its first consumer and passes `formal-ai-source: source`, because a change to the meta algorithm must be measured by the branch making it. link-assistant/hive-mind#2233 asks for the first outside installation, on `issues: opened`.
- Issue #1107: `.github/actions/formal-ai-binary` provides `target/release/formal-ai` from a cache keyed by the content of the sources it is built from. Seven workflows compiled the same release binary on every push, two to five minutes each; a push that changed no source now reuses the build and installs no Rust toolchain.

### Changed
- Issue #1107: the pipeline's heavy jobs (the macOS archive, the Docker image check, the six box-image legs and the 25-minute agent CLI end-to-end run) are gated on a new `pipeline-changed` output instead of `workflow-changed`. The old flag was true for any file under `.github/workflows/`, so editing a scheduled benchmark bought the full cost of a code push; the new one is true only for the pipeline's own definition, a composite action it calls, or a script those run.
- Issue #1085: the CI contract tests read `.github/actions/**` as well as `.github/workflows/**`. The shell CI executes is the same shell whichever directory it sits in, so moving a `git push` or a credentialed checkout into a composite action must not move it out of review.
