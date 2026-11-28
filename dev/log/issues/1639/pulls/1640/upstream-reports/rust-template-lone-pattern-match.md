# download-artifact adapter: a pattern that matches exactly one artifact is extracted flat, without its `<name>/` folder

## Summary

`.github/actions/download-artifact` calls `actions/download-artifact@v8.0.2`
with `skip-decompress: true`, and then `scripts/extract-artifacts.py` extracts
the raw ZIPs. The adapter expects one folder per artifact for a `pattern:`
selection without `merge-multiple` (`workflows.yml` checks
`artifact-separate/extraction-one/check.bin`). It gets those folders only when
the pattern matches **two or more** artifacts.

When the pattern matches exactly one artifact, upstream writes the raw archive
straight into `path` as `<artifact-name>.zip`. Upstream does this whenever
`isSingleArtifactDownload || mergeMultiple || artifacts.length === 1` in
`src/download-artifact.ts`, behaviour added in fff8c148a (v5). The extractor
then takes the `download.is_dir()` false branch and writes the members into the
destination root. A consumer that globs `<path>/*/file` then sees nothing.

Upstream tracks this, with no fix released:

- actions/download-artifact#426
- actions/download-artifact#455
- unmerged PR actions/download-artifact#422

## Reproduction

1. In `.github/workflows/workflows.yml` (artifact round trip), add:

   ```yaml
   - uses: ./.github/actions/download-artifact
     with:
       pattern: extraction-one*
       path: artifact-lone
   - run: cmp artifact-fixture/one/check.bin artifact-lone/extraction-one/check.bin
   ```

2. `cmp` fails: `artifact-lone/extraction-one/check.bin` does not exist, and
   `artifact-lone/check.bin` does.

No template consumer breaks today:

- `desktop-release.yml` reads `release-assets` recursively (`find`, `**/*`);
- `release.yml` merges.

A derived repository hit it, though. In link-assistant/formal-ai, a one-leg
ladder run downloaded its legs by pattern. The leg passed, and the join job
reported "the plan has 1 shard(s), 0 wrote a result" (formal-ai#1639,
run 45659315559).

## Workaround

Pass `merge-multiple: true` and read the files recursively, or select by
`name:` when exactly one artifact is expected.

## Suggested fix (applied in formal-ai#1640)

In the adapter, when the selection is a pattern (or every artifact) and
`merge-multiple` is false, and the raw directory holds a single file
`<name>.zip`, extract it under `<name>/`. Refuse a lone raw file whose name
does not end in `.zip`, because that name carries no artifact name. Keep a
selection by `name:` flat, as upstream documents.

```python
def nest_single_match(sources):
    if len(sources) != 1 or sources[0][0].parts:
        return sources
    _, archive = sources[0]
    if not archive.name.endswith(".zip"):
        raise ValueError(f"A single pattern match carries no artifact name: {archive.name}")
    return [(member_path(archive.name[:-4]), archive)]
```

The adapter passes `--nest-pattern-selection ${{ inputs.name == '' }}`.

Add a lone-match case to `experiments/issue-223-artifact-tests.py` and to the
round trip in `workflows.yml`.
