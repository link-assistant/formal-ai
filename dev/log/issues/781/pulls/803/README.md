# Issue 781 / pull 803 evidence bundle

This directory preserves the evidence used to reproduce, diagnose, fix, and
verify [formal-ai#781](https://github.com/link-assistant/formal-ai/issues/781)
in [PR #803](https://github.com/link-assistant/formal-ai/pull/803). Raw material
is retained alongside concise analysis so a reviewer can audit conclusions
without rerunning third-party clients or depending on mutable GitHub pages.

## Conclusions

- PR #795 implemented multi-source research, recursive follow-up, normalized
  shared-dialog ingestion, and a generic Links-based option network, but the
  July 19 reproduction showed its transport UX was still silent and could end
  without a usable response.
- The immediate stall was not one defect. Batched tool plans contained no
  user-facing narration; open-world requests could route to local grep;
  Responses namespace children were invisible to the planner or returned in an
  envelope Codex could not dispatch; and Codex transport wrappers polluted the
  URL/result data used by later turns.
- Formal AI now emits useful localized text before every research action, plans
  one search/fetch per turn, retains successful evidence by URL, and finishes
  with a cited synthesis. The same behavior is produced through Chat
  Completions, Responses, Anthropic Messages, and Gemini.
- Exact request/response bodies can now be recorded per dialog as JSONL by
  setting `FORMAL_AI_DIALOG_LOG_DIR`. Logging is off by default because bodies
  can contain private prompts and tool output.
- The reusable E2E drives the exact Russian prompt through Agent, OpenCode,
  Claude, and Codex. Each client performs one search, three separately narrated
  fetch turns, and a final synthesis. The deterministic MCP fixture isolates
  protocol behavior; separate native-network runs remain here as evidence of
  real provider and anti-bot variability.

## Analysis

- `analysis/timeline.md`: chronological reconstruction.
- `analysis/requirements.md`: exhaustive requirement-to-evidence trace.
- `analysis/root-causes.md`: each independently reproduced failure and its fix.
- `analysis/solution-plans.md`: alternatives, chosen design, remaining limits.
- `research/online-research.md`: official specifications, related components,
  known upstream defects, and source URLs.

## Raw evidence map

- `github/`: issue, comments, events, PR #795 and #803 records, diffs, screenshots,
  strict PNG structure/CRC validation, the 67,000-line reference AI session,
  and its compact event timeline.
- `ci-logs/`: historical branch run metadata and the complete 2,662-line log.
- `real-cli/`: native Agent/OpenCode/Claude/Codex experiments, including every
  failed intermediate Codex namespace/approval/normalization run.
- `real-cli/four-client-final-2/`: final four-client proof. Each client directory
  contains `client.log`, `formal-ai.log`, and `dialogs/*.jsonl`.
- `tests/`: focused red/green test logs and the final harness summary.
- `research/upstream/`: related Agent CLI and Codex issues plus the exact public
  Codex report submitted during this investigation.
- top-level `reproduction-*-before-fix.log` / `*-after-fix.log`: minimum
  reproductions for narration, action sequencing, intent routing, namespace
  visibility, namespace dispatch, and transport-envelope normalization.

## Fastest audit path

1. Read `github/issue-view.txt` and the two validated PNG screenshots.
2. Compare `reproduction-before-fix.log` with `reproduction-after-core-fix.log`.
3. Read `tests/four-client-harness-final-2.log`.
4. Inspect one final dialog file, for example
   `real-cli/four-client-final-2/codex/dialogs/dialog_6e49014bed1c0684.jsonl`.
5. Follow the root-cause rows to the focused unit/integration logs.

## Evidence qualifications

The `.example.test` sources in the four-client harness are deterministic test
fixtures, not live product listings. Native network runs and browser captures
are preserved separately. Amazon returned an automated-access page, Google AI
Mode exposed no replayable transcript, and live search result breadth varied.
Accordingly, the product recommendation remains conditional and the software
tests verify research mechanics—not current stock, polarity, or seller claims.

Files ending in `.failed` or `.stderr` preserve transient GitHub API failures;
their corresponding GraphQL or successful REST captures are present. They are
kept so the collection process itself remains auditable.
