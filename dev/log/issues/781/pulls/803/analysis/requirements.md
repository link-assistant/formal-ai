# Exhaustive requirement trace

This matrix combines the original issue, both issue comments, the review that
expanded PR #795, and the current solver instructions. “Qualified” means the
implementation meets the software requirement while documenting an external or
non-deterministic limit rather than inventing evidence.

| ID | Requirement | Status and evidence |
|---|---|---|
| R1 | Answer the exact Russian A325-45/Amazon India request as well as available evidence permits. | Met, qualified: `docs/case-studies/issue-781/recommendation.md`; exact prompt is fixed in the E2E harness. Live stock/polarity remains unverified. |
| R2 | Search for an authentic/original part first, then official-compatible, then generic-compatible. | Met: `src/option_network.rs`, `Tier::LADDER`, option-network tests. |
| R3 | Allow a solution made from two separate purchases, such as supply plus conversion adapter. | Met: minimal satisfying candidate sets and `two_separate_items_form_one_plan_when_neither_suffices_alone`. |
| R4 | Return every minimal compatible option, cheapest first—not only one recommendation. | Met: fixed-point price ranking and `cheaper_options_are_listed_first_and_bundles_are_not_padded`. |
| R5 | Perform real search plus page fetch/capture; do not trust snippets alone. | Met, qualified: native captures in the case study and `real-cli/`; blocked/empty pages are explicitly untrusted. |
| R6 | Verify source contents and retain the exact source-to-evidence relationship. | Met: per-URL successful fetch records, independent source rendering, and failed-source regression. |
| R7 | Use recursive, multi-turn search/fetch/refine reasoning with termination. | Met: one action per turn, deeper-round planner, round budget, no-repeat guards, R15/R16 tests. |
| R8 | Represent constraints, facts, options, and sources as associative Links, not a learned score. | Met: `world_model::Context`, `OptionNetwork::links_notation`, R17 tests. |
| R9 | Generalize beyond chargers/products/languages and avoid issue-specific production vocabulary. | Met: generic units/constraints and unrelated enlarger-lens test; issue words appear only in tests/docs. |
| R10 | Populate options from fetched evidence, abstaining when an attribute is absent. | Met: `option_evidence::candidate_from_page` and multilingual/part-number regressions. |
| R11 | Capture and convert the supplied ChatGPT shared dialog to Links Notation. | Met: 35-turn capture, byte-identical direct/adapter `.lino`, unit and CLI integration tests. |
| R12 | Prepare a Google AI Mode adapter without guessing unavailable text. | Met, qualified: normalized adapter support exists; browser capture preserves `no_transcript_in_captured_dom`. |
| R13 | Download and preserve all relevant issue/PR/log/image data. | Met: this evidence directory and the case-study raw-data inventory; screenshots were downloaded, identified as valid PNGs, then visually inspected. |
| R14 | Reconstruct the sequence of events. | Met: `timeline.md` and raw API timelines. |
| R15 | List every requirement and trace it to evidence. | Met: this matrix and the case-study requirement matrix. |
| R16 | Find the actual root cause of every observed problem. | Met: `root-causes.md`; independent red/green evidence exists for every in-repository defect. External limits are separately labelled. |
| R17 | Propose alternatives and plans for each requirement/root cause. | Met: `solution-plans.md`. |
| R18 | Search online for additional facts and existing components/libraries. | Met: `../research/online-research.md`, using official API/protocol/vendor sources where available. |
| R19 | Add opt-in verbose/debug output when evidence is insufficient, off by default. | Met: `FORMAL_AI_TRACE_REQUESTS` plus new `FORMAL_AI_DIALOG_LOG_DIR`; exact bodies are opt-in. |
| R20 | Report actionable related-project defects with repro, workaround, and code suggestions. | Met: OpenAI Codex #14242 comment 5018114084 and Agent #194 comment 5018466226. The existing OpenCode #20465 report already supplies its reproduction, workaround, root cause, and code fix, so it is retained and linked rather than duplicated. |
| R21 | Apply the behavioral fix across the entire agentic API codebase. | Met: Chat Completions, Responses, Anthropic Messages, and Gemini all share the planner and have protocol-specific assertions. |
| R22 | Split work into smaller actions rather than a silent multi-action response. | Met: one planned research action per server turn; `independent_sources_are_fetched_in_separate_agent_turns`. |
| R23 | Explain why/what is happening before every tool call. | Met: localized narration precedes the tool call/block/part on all four protocols; fetch narration names the URL. |
| R24 | Explanations must be useful and natural rather than robotic or private chain-of-thought dumps. | Met: concise first-person action/target narration; private hidden reasoning is not exposed. Protocol tests assert ordering and target content, not brittle exact prose. |
| R25 | Produce a final summary after all actions. | Met: final planner turn renders every successful cited source after search and separate fetches. |
| R26 | Avoid more than ten seconds of silent “thinking” per message. | Met by design, qualified: actions are externally visible separate turns; no wall-clock SLA is asserted because client/network latency is outside the server. Per-turn timestamps remain in dialog logs for latency diagnosis. |
| R27 | Preserve exact server logs per dialog, including every request, response, and action. | Met: `src/dialog_log.rs`; atomic JSONL append at the common HTTP boundary with timestamp, dialog/request IDs, method/path/status, normalized tool summaries, and exact request/response bodies. Authentication headers are deliberately not copied into the record. |
| R28 | Keep exact dialog logging disabled by default for privacy. | Met: no file is written unless `FORMAL_AI_DIALOG_LOG_DIR` is configured; docs warn that bodies may be sensitive. |
| R29 | Full E2E with Agent CLI. | Met: final harness records 1 search / 3 fetch / 5 turns; up to three clean-session retries cover known Agent early-exit variability without weakening assertions. |
| R30 | Full E2E with OpenCode. | Met: final harness records 1 / 3 / 6 with visible Russian narration. |
| R31 | Full E2E with Claude Code. | Met: final harness records 1 / 3 / 5 through a local tool-only MCP server. |
| R32 | Full E2E with Codex CLI. | Met: final harness records 1 / 3 / 5 and validates Responses namespace dispatch. |
| R33 | Preserve full client, server, and per-dialog logs for every final E2E. | Met: `../real-cli/four-client-final-2/{agent,opencode,claude,codex}/`. |
| R34 | Put the reusable four-client regression in CI. | Met: `.github/workflows/release.yml` installs all clients and runs `experiments/agent_cli_e2e/run_issue_781.sh`. |
| R35 | Reproduce each logic bug before fixing it and retain red/green proof. | Met: commits start with `361c7a83`; all later routing/namespace/envelope red and green logs are preserved. |
| R36 | Preserve raw transport output even when the planner needs normalized content. | Met: `raw_tool_result_is_retained_exactly_for_durable_recording` plus separate `normalized_payload`. |
| R37 | Prefer client-executable MCP research tools over hosted fallbacks. | Met: Responses namespace integration test; hosted tool behavior remains covered. |
| R38 | Keep the solution one generalized meta-algorithm rather than four client-specific planners. | Met: shared intent/capability/planner/progress/result logic; adapters only translate each protocol envelope. |

## Explicit qualifications

- A separate visible action reduces silent periods but cannot guarantee a
  third-party CLI renders within ten seconds on every network. Exact timestamps
  now make such a future failure diagnosable.
- The four-client CI uses deterministic fetched pages so failures identify
  protocol regressions. Native internet evidence is intentionally separate
  because search rank, authentication, and anti-bot pages are mutable.
- Formal AI can ingest Google normalized output, but it cannot bypass a Google
  challenge or fabricate a transcript that the capture provider did not expose.
- No purchase is executed. Current availability and electrical safety require
  live seller/manufacturer confirmation.
