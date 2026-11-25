# Issue 1028 ladder: a passing leg joined as "0 wrote a result"

## Timeline

- 2026-11-25 16:56 UTC: run 45659315559 dispatched on `main` 3388db348, depth 5,
  `NODE_FILTER=2.2.2.2.1`, to confirm the ladder fix 971802a07.
- Leg `depth-5-2.2.2.2.1` passed: `failures "0"`, `leaf_nodes_passing "1"`
  (`ci-logs/round1/ladder-45659315559-leg-ladder-result.lino`).
- 17:29 UTC: "Join the legs into one run" failed:
  `the plan has 1 shard(s), 0 wrote a result`
  (`ci-logs/round1/ladder-45659315559-compare-job.log`).

## Root cause

`combine-shards.py` globs `<legs>/*/ladder-result.lino`, so it expects one
folder per leg. The download step selects by `pattern:` without
`merge-multiple`. Since actions/download-artifact fff8c148a (v5), a selection
that matches exactly one artifact is written straight into `path`
(`isSingleArtifactDownload || inputs.mergeMultiple || artifacts.length === 1`
in `src/download-artifact.ts` at v8.0.2). That holds for a pattern too, which
the upstream README does not document.

The local wrapper `.github/actions/download-artifact` runs v8.0.2 in raw mode.
Its job log shows `Downloading raw file (non-zip) to:
.../artifact-archives.5YKHUP/issue-1028-agent-cli-binary-tree-depth-5-2.2.2.2.1.zip`,
flat. `scripts/extract-artifacts.py` reproduced the upstream layout, so the
leg's files landed at the destination root.

A full ladder dispatch matches many legs and never showed it. A one-leaf
filter always fails.

## Same defect elsewhere

`experiments/issue-1639/list-unmerged-pattern-downloads.mjs` lists the six
wrapper calls that select by pattern without merging:

| Workflow | Pattern | Reader | With one match |
| --- | --- | --- | --- |
| issue-1028-agent-ladder.yml | `issue-1028-agent-cli-binary-tree-*` | combine-shards.py `*/ladder-result.lino` | fails (this run) |
| issue-1028-agent-ladder.yml | `issue-1028-agent-cli-binary-tree-depth-5-*` | combine-shards.py | fails |
| coverage.yml | `browser-coverage-shard-*` | `readBrowserDurationPackets`: one folder per shard | throws "Unknown browser artifact folder" |
| desktop-release.yml | `native-receipt-*` | `native-release-evidence.mjs`: folders by prefix | drops the receipt |
| coding-ladder.yml | `coding-ladder-shard-*` | `MERGE_FROM` at any depth | works |
| regenerate-self-ast-census.yml | `source-content-part-*` | recursive `receiptFiles` | works |

## Fix

The wrapper's extractor restores the `<artifact-name>/` folder from the raw
`<name>.zip` for pattern and every-artifact selections without
`merge-multiple`. It refuses a lone archive whose file name carries no artifact
name. Selection by name or by ID stays flat, as upstream documents. One fix
covers all six consumers and future ones.

Tests:

- `experiments/issue-1580/artifact-extraction-fixtures.py`
  `test_lone_pattern_match_keeps_its_artifact_folder` (real ZIP bytes).
- `rust/tests/web/native-test-artifact.test.mjs` "issue #1639: a lone pattern
  match extracts under its artifact name, as two matches do". This runs in the
  pull request lane and is recorded in `data/meta/main-only-regressions.lino`.

## Upstream

Already reported, so no new report was filed:

- https://github.com/actions/download-artifact/issues/426 (open since
  2025-08-30, 13 comments);
- https://github.com/actions/download-artifact/issues/455;
- fix PR https://github.com/actions/download-artifact/pull/422, unmerged.

A comment on #455 proposes the same `skip-decompress: true` workaround the
wrapper builds on.
