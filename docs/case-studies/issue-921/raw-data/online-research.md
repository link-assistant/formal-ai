# Upstream And Prior-Art Survey

The implementation uses the projects' own public contracts and source rather
than a substitute protocol:

- Hive Mind issue #2059 defines the blocked
  `--tool agent --model formal-ai` entry point.
- Hive Mind PR #2108 adds Formal AI dispatch across the supported agent tools.
- Hive Mind PR #2147 is the most recent implementation and makes the native
  Agent CLI plus on-demand Formal AI runtime the reachable path.
- Formal AI issue #655 preserves the earlier inner-loop-only result.
- Formal AI issue #703 supplies the external-CLI session and replay contract.
- Formal AI issue #916 supplies E69's coding-ladder foundation.

The corresponding GitHub API responses are preserved under `github/`, including
empty comment/review arrays where no feedback existed at capture time.
