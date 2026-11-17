# PR 1585 development and CI record

[PR 1585](https://github.com/link-assistant/formal-ai/pull/1585) remains a draft.
All seven issue acceptance criteria remain pending; see the
[issue case study](../issue-1584/README.md).

The initial head was `552ffaf803ea4399dbe4806f8bfc3efd8797eecf`. Eleven
workflow runs created at 2026-11-17T11:37:01Z completed as `action_required`
without jobs. Log retrieval returned no available logs. Raw run metadata
and each log retrieval error are retained in the issue evidence archive.
These results are not passing CI and are not evidence of source failures.

The implementation commit is `409d72d08`. It adds experimental selectors,
lossless opaque retention and one declared proposition with five localized
realizations. Source generation was checked, but native execution remains
unverified. The next push must be reviewed against current-head CI.

Local checks: 86 gate passes, 52 CI-only skips, no gate failures; focused
JavaScript tests 10 + 3 + 7 passed. The raw archive also keeps earlier failures.
No code or documentation is claimed to have been autonomously authored by
Formal AI: supplied instructions and scripts count as zero accepted units.
