# CI/CD evidence for issue #808

Excerpts from the failing default-branch runs. The complete logs are 2–4 MB
each and are not committed; re-download them with:

```bash
gh run view <run-id> --repo link-assistant/formal-ai --log > run-<run-id>-full.log
```

| File | Run | Workflow | Commit | What it shows |
| --- | --- | --- | --- | --- |
| `run-29723467001-auto-release.log` | [29723467001](https://github.com/link-assistant/formal-ai/actions/runs/29723467001) | CI/CD Pipeline | `76bdb6b5` | Auto Release aborts: `no committed Formal-AI-Evidence in 10e65ae2 … records session issue-804-claude-20260720` |
| `run-29719602956-excerpt.log` | [29719602956](https://github.com/link-assistant/formal-ai/actions/runs/29719602956) | CI/CD Pipeline | `8b5acee0` | Same failure, previous commit |
| `run-29724500254-macos-codesign-excerpt.log` | [29724500254](https://github.com/link-assistant/formal-ai/actions/runs/29724500254) | Desktop Release | `76bdb6b5` | macOS ad-hoc signing fails on the bundled Chrome for Testing framework |
| `run-29720321919-macos-codesign-excerpt.log` | [29720321919](https://github.com/link-assistant/formal-ai/actions/runs/29720321919) | Desktop Release | `8b5acee0` | Same failure, previous commit |

## Why the macOS excerpts are filtered the way they are

The filter keeps every `##[error]`, `⨯`, `failedTask`, `unsealed`,
`executing custom sign`, `[adhoc-sign-mac]` and `Skipped...` line, plus a sample
of the `Signing... .../browser-runtime/...` lines. Three facts follow from it:

1. **1506 paths under `Contents/Resources/browser-runtime` were signed
   individually.** The exclusion in `desktop/scripts/adhoc-sign-mac.cjs` was
   therefore not in effect.
2. **Not a single `[adhoc-sign-mac]` line appears**, even though the step sets
   `FORMAL_AI_MACOS_SIGN_DEBUG: 1` — the hook's diagnostics never reached the
   log. (Confirmed against the *complete* log, not just the excerpt.)
3. **Not a single `Skipped...` line appears** either — `@electron/osx-sign`
   logs that for every path its `ignore` predicate rejects.

See `../analysis.md` §4 for the resulting root cause and fix.
