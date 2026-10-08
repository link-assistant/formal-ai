## Problem

A pull request that changes agentic routing gets **no four-client coverage**. The multi-client E2E scenarios are gated on `full-replay`, which `.github/workflows/release.yml:1197` sets true only for `main`, `workflow_dispatch` and `schedule`:

```yaml
full-replay: ${{ github.ref == 'refs/heads/main' || github.event_name == 'workflow_dispatch' || github.event_name == 'schedule' }}
```

On a feature branch the step reports `skipped`, so the first time the scenario actually runs is **after merge**, on `main`.

## Why it matters

This has now produced two consecutive broken pipelines on `main`:

- #1134 was green as a pull request. The four-client #781 scenario ran only after merge and failed (`claude never reached websearch`) — [run 34798166044](https://github.com/link-assistant/formal-ai/actions/runs/34798166044).
- The fix (#1135, for #1136) likewise could not be verified in CI. Every run of that PR skipped the scenario; it had to be verified by running all four real CLIs locally.

The blind spot is specific: `src/agentic_coding/capability_router.rs` decides which tool each client is asked to call, and the clients differ precisely in which tools they advertise and permit. `agent` and `opencode` passed the same run where `claude` failed, so single-client coverage cannot catch it.

Note the workflow_dispatch escape hatch is not usable either: `release.yml`'s dispatch requires `bump_type` and drives a release, so it is not a way to replay E2E on a branch.

## Suggested fix

Run the multi-client scenarios on a pull request when the change touches the code that decides routing. Options, cheapest first:

1. Set `full-replay: true` when the PR's diff touches `src/agentic_coding/` (`detect-changes` already computes path filters), leaving other PRs on the reduced set.
2. Add a small always-on four-client gate covering just the tool-precedence axis — the `run_issue_781.sh` clients with one scenario rather than all of them.
3. Make the `agent-cli-e2e.yml` workflow independently `workflow_dispatch`-able with a `full-replay` input, so a branch can be replayed on demand without a release.

The green-ledger digest already covers `src`, so a routing change would correctly invalidate a cached green result once the scenario is allowed to run at all.

## Acceptance

A pull request that changes `src/agentic_coding/capability_router.rs` fails in CI if any of the four clients (`agent`, `opencode`, `claude`, `codex`) stops reaching its search, fetches and cited synthesis — before the change lands on `main`.

