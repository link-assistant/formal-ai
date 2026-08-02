# Issue #804 — CI/CD false positives, false negatives, warnings and errors

Evidence bundle for [issue #804](https://github.com/link-assistant/formal-ai/issues/804) /
[PR #805](https://github.com/link-assistant/formal-ai/pull/805).

## Contents

| Path | What it is |
| --- | --- |
| `runs-main.json` | `gh run list --branch main --limit 15` snapshot taken 2026-07-20T06:20Z |
| `ci-logs/failed-29719602956.log` | Failed-step log of `CI/CD Pipeline` run 29719602956 (`8b5acee`) |
| `ci-logs/failed-29720321919.log` | Failed-step log of `Desktop Release` run 29720321919 (`8b5acee`) |
| `ci-logs/jobs-*.json` | Job/step metadata for both runs |
| `analysis.md` | Timeline, requirement list, root-cause analysis, solution plans |

Reproduce the collection with:

```bash
gh run list --branch main --limit 15 --json databaseId,name,conclusion,status,createdAt,headSha
gh run view <run-id> --log-failed > ci-logs/failed-<run-id>.log
gh run view <run-id> --json jobs > ci-logs/jobs-<run-id>.json
```
