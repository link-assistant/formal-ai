# Hosted worker receipt regeneration

Layered CI run 44938456935 rejected stale worker inputs after the native source
partition. The checked-in worker must be rebuilt from the actual selected source;
changing its manifest without compiling would manufacture evidence.

The default-branch build policy now includes one explicit, read-only manual
maintenance exception. A Layered CI dispatch with mode `regenerate-translations`
and target `worker-receipt` selects exactly one job. Ordinary pull requests,
pushes and check dispatches do not allocate it. The job builds on a hosted runner,
verifies the actual selected compiler receipt and manifest, and uploads the
worker, receipt and manifest for one day. The manifest keeps only the actual
compiler dependency closure; package/version and build-recipe metadata stay bound
in the producer receipt. This preserves existing drift semantics when a release
changes package metadata without changing worker inputs. It has no publication, deployment or
cache-writing authority. Checkout, repository, source SHA, event, run and attempt
are checked before compilation. Its ten-minute job and 420-second command budget
are finite. No local Rust compilation is used.

The user's standing authorization to repair CI covers this documented workflow
and policy repair; no separate approval is required. The earlier inactive
proposal has been archived as evidence and replaced by the concrete job.

The job must first exist on the branch before GitHub can run it. This creates a
bootstrap round: push the complete source repair batch, dispatch hosted
regeneration, download and verify its actual artifact, then commit the generated
files together with any other actionable failures from that head's terminal CI.
The stale-worker check may fail in that prerequisite round. It remains an open
failure until the regenerated files pass the unchanged check. Hosted build and
artifact verification have not yet run at the time of this draft.

`experiments/issue-1580/worker-receipt-regeneration.test.mjs` checks compiler dependency/metadata separation, negative
admission, exact job permissions and budget, and complete job selection on both
main and this prepared branch. Supplied code and conclusions earn zero
independent Formal AI authorship or reasoning credit.

All four focused JavaScript cases and the 331-case maintained entry pass locally.
These are admission and manifest regressions, not evidence of hosted compilation.

Pull request: https://github.com/link-assistant/formal-ai/pull/1581.

## Verified hosted result

[Run 44962282997](https://github.com/link-assistant/formal-ai/actions/runs/44962282997) compiled exact source `de79e6ece67142fe8c6012059e6eb2bc0300e7bd` with `rustc 1.100.0`. The complete selected-source verifier validates the source tree, all 27 ordinary input files, protocol, compiler arguments and producer identity. The downloaded three-file archive matches the Actions SHA-256 digest. Its 25-input compiler manifest and 311,765-byte worker pass the unchanged currency check after application. No local Rust compiler ran. Full branch CI and the broader issue criteria remain separate.
