# Architecture: Self-Development Release Loop

Part of the [architecture overview](../../ARCHITECTURE.md). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## Formal AI self-development release loop

Issue #924 composes the existing session-backed self-hosting metric with the
normal GitHub merge topology. A contribution is still attributed from its
committed session and evidence trailers; an additional canonical PR trailer is
eligible for the release floor only when a matching first-parent GitHub merge
contains that exact non-merge commit on its second-parent branch. This keeps a
direct or rewritten claim from masquerading as reviewed work.

Before version mutation, the release path requires one qualifying PR and
projects the existing three-release changed-line share. The next target is
`max(previous target, previous comparable trailing share)`, so the ledger is a
monotonic floor as well as a report. Each new row records the target and all
qualifying PR URLs. A failed floor leaves the release range open for additional
reviewed work instead of writing an impossible historical row.

This layer observes merged ancestry; it does not add an AI-only authority.
Review, CI, and the #656 trusted promotion protocol remain outside and above
the authorship measurement. E69 supplies the write-effect foundation and E74
supplies the replayable Hive Mind/Agent CLI route used to produce candidates.
