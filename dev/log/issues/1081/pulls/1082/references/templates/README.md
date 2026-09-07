# Template snapshots used by issue #1081

Only the commit each template was read at is recorded here, not the trees. The
five commits are **byte-identical to the ones issue #1079 snapshotted**, so the
files themselves are already in this repository at
[`../../../../1079/pulls/1080/references/templates/`](../../../../1079/pulls/1080/references/templates/)
and copying them again would add ~200 duplicate files that no reader can tell
apart from the originals.

| template | commit | upstream default branch at the time of this analysis |
|---|---|---|
| rust | `4d444d976f6bb46168b9f4a7001c775bf7c0ef6b` | same commit |
| js | `338fafa7b428637a18fe06a114d446a5cfb23ae7` | same commit |
| python | `81c978684f04ce018f606e5fcd418202157e009d` | same commit |
| php | `5c3c906c1d8b30941f639159ebb9bf0657616fdb` | same commit |
| csharp | `83efb9e4482ffed9c9ef1c7b07ea250ac6d3b141` | same commit |

The third column is the reason this round could reuse the previous round's
trees rather than re-fetching them: none of the five templates moved between
issue #1079 and issue #1081, so "the snapshot" and "upstream today" were the
same thing while every measurement below was taken.

## Reproducing the check

```bash
for t in rust js python php csharp; do
  r="link-foundation/$t-ai-driven-development-pipeline-template"
  echo "$t $(gh api "repos/$r/commits/main" -q .sha) $(cat "$t-template.HEAD")"
done
```

Both columns matched for all five when the reports in `../../upstream-reports/`
were filed. If they stop matching, re-run the measurements before trusting the
counts in those reports -- every one of them is a count over these trees.
