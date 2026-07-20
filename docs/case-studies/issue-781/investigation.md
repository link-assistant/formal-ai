# Investigation, timeline, and design

## Timeline

- 2026-07-18 21:00 UTC: issue #781 was opened with two shared-dialog URLs and two fallback Amazon URLs.
- 2026-07-19 06:18 UTC: draft PR #795 and the prepared branch were created.
- 2026-07-19 06:20–06:29 UTC: repository guidance, issue history, PR feedback endpoints, issue #552, recent browser research work, and upstream issues were reviewed.
- 2026-07-19 06:29 UTC: existing shared-dialog tests passed (7 tests). The first clean desktop `npm ci` failed because the committed lockfile lacks two platform packages and the environment initially supplied Node 20 while current Electron packages require Node 22; this is recorded, not hidden, in `npm-ci-baseline.log`.
- 2026-07-19 06:31 UTC: Chromium captures completed. ChatGPT returned 35 normalized turns. Google static capture hit a challenge; the browser fallback loaded but yielded `no_transcript_in_captured_dom`. Both supplied Amazon product pages returned the automated-access notice.
- 2026-07-19 06:33 UTC: the new whole-task regression failed because the planner emitted one fetch rather than three.
- 2026-07-19 06:36 UTC onward: bounded fan-out, URL/result association, normalized capture JSON parsing, parity artifacts, and tests were added.
- 2026-07-19 08:13 UTC: the final optimized-binary Agent CLI replay passed
  its evidence boundary: one live search, three planned fetches, and all three
  fetches executed by the client (nine lifecycle events in its event stream).
- 2026-07-19 18:26–18:36 UTC: issue #781 was reopened with screenshots showing
  OpenCode waiting after Grep and later ending with a no-content error. Review
  required one narrated action per turn, final synthesis, exact per-dialog logs,
  and E2E proof in Agent, OpenCode, Claude, and Codex.
- 2026-07-20 00:31–01:37 UTC: red/green tests isolated silent/batched actions,
  open-world-to-grep misrouting, Responses namespace discovery/dispatch, missing
  MCP safety annotations, and Codex result-envelope pollution.
- 2026-07-20 01:49 UTC: the reusable deterministic harness passed all four
  native clients. Each completed one search, three separate fetches, and a final
  cited synthesis; exact client, server, and dialog JSONL logs are preserved.
- 2026-07-20 01:56 UTC: the Codex namespace reproduction, workaround, and fix
  suggestions were posted to OpenAI Codex issue #14242.

## Code trace and root causes

### Research stopped too early

`agentic_coding::web_research::plan_web_research_step` previously called `preferred_url`, produced one fetch call, and finalized as soon as `Progress` observed that fetch. That policy is adequate for a simple date lookup but structurally incapable of compatibility research: one page cannot independently establish the device requirement, connector geometry/polarity, and a seller listing.

`Progress` also retained only the latest successful fetch body. Even if a caller supplied several fetch results, earlier evidence and the URL-to-body relationship were lost.

The evidence model is deliberately bounded rather than a crawler:

1. extract and de-duplicate search-result URLs;
2. place the first government/education source first when one exists, preserving search order otherwise;
3. retain at most three URLs;
4. emit one fetch per agentic turn with user-visible narration;
5. recover each fetch URL from its tool-call arguments and pair it with that result;
6. extract query-relevant text independently and cite its exact URL.

The release-server Agent replay exposed an external transport constraint: Agent
CLI 0.25.0 can record an `unknown` finish reason and exit before requesting the
final answer (upstream issue #249). Both parallel and one-fetch controls showed
the boundary, so batching was not its sole cause. The final CI harness retries
Agent with a fresh session at most three times but still requires the complete
synthesis; partial runs remain preserved as evidence.

No Acer, charger, Amazon, voltage, language, or marketplace vocabulary was added to planner code.

### The reopened UX failure crossed protocol boundaries

The exact Russian prompt initially routed to grep when the client advertised a
local search tool. Research responses could then contain calls without assistant
text and batch several fetches, leaving the user without useful progress. The
shared planner now prefers research for open-world finds, emits exactly one
action per turn, and derives concise localized narration from the tool and its
query/URL.

Each protocol preserves that ordering natively: Chat assistant content precedes
`tool_calls`; Responses output contains a message item before the function call;
Anthropic emits a text block before `tool_use`; Gemini emits a text part before
`functionCall`. This is user-facing action rationale, not hidden chain of thought.

Codex exposed two more boundaries. Its MCP functions arrive beneath a Responses
namespace, while its router requires a call to round-trip as separate namespace
and child name. Formal AI now recursively exposes namespace children to the
planner and rehydrates the selected identity in output. Known Codex MCP wrappers
are normalized only for planning; raw tool content is retained unchanged.

### Exact dialog observability was missing

Request traces could prove that an endpoint was called but could not reconstruct
the exact request, response, and action sequence reported in issue #800. With
`FORMAL_AI_DIALOG_LOG_DIR`, the common server boundary now appends timestamped,
sequenced request/response exchanges to a stable per-dialog JSONL file. The
feature is off by default and documented as potentially sensitive.

### Capture and conversion were disconnected

Formal AI issue #552 added direct ChatGPT HTML and compact Markdown parsing, but intentionally deferred browser/provider capture to `link-assistant/web-capture#141`. That upstream issue is now implemented in `@link-assistant/web-capture`: it provides `shared-dialog`, tries static HTTP then browser rendering, and emits normalized JSON plus structured unsupported diagnostics.

Formal AI had no parser for that normalized boundary, so users could capture or convert but could not connect the two without an ad-hoc transformation. `SharedDialogFormat::WebCaptureJson` now consumes the maintained contract and exports the same local `demo_memory` representation. Unsupported provider results remain errors containing provider, reason, and message.

This is preferable to duplicating selectors and anti-bot behavior in Rust. `link-foundation/meta-language#168` was reviewed too; maintainers marked a new global schema out of scope, so this change consumes the web-capture contract rather than reopening that design.

## Actual source observations

- The ChatGPT share is healthy, titled `Зарядка для Acer Aspire`, and contains 35 visible user/assistant turns. It records the laptop-label observation `12 V / 2 A`, the official-page `24 W` evidence, uncertainty around the barrel size, an incorrect detour into 19 V Acer adapters, and finally the absence of a confirmed live exact Amazon listing.
- Acer's indexed product page identifies the A325-45 supply as 24 W. The transcript's photograph supplies the more specific 12 V / 2 A marking. These agree arithmetically, but the official indexed page does not establish connector size.
- Search indexing found a marketplace title that names A325-45, 12 V, 2 A, and 3.5 × 1.35 mm. Its live rendered page stayed at a loading shell, so it is corroboration, not authoritative electrical evidence.
- Project-native DuckDuckGo capture found Amazon ASIN `B0CG66QYWN`, whose indexed title says Tonton 12 V, 2 A, 24 W and whose URL identifies a 3.5 × 1.35 mm interchangeable tip. This is a substantially better candidate than joining two adapters, but it does not name the A325-45 and the indexed result did not establish polarity or inventory.
- Google did not expose replayable content in static or browser-rendered capture. The share must not be represented as an empty or guessed transcript.
- Amazon returned a bot page for both issue URLs and the newly found Tonton candidate. Listing title fragments and search snippets are insufficient to verify polarity, inventory, or all seller claims.

## Libraries and related work

- `@link-assistant/web-capture` supplies Chromium-backed page capture and the shared-dialog normalized contract.
- `@link-assistant/web-search` supplies the desktop's Google/Bing/DuckDuckGo providers and reciprocal-rank fusion.
- `serde_json` decodes normalized adapter JSON and tool-call arguments.
- Existing Formal AI `MemoryEvent`/Links Notation export remains the sole replay representation.
- Recent PR #766 established permission-free read-only browser search/fetch; PR #793 established query-relevant fetched-page extraction. This change composes those capabilities instead of replacing them.
- OpenAI, Anthropic, and Gemini protocol documentation all support ordered text
  and tool-action structures or compositional multi-turn function calling.
- MCP tool annotations describe read-only/open-world behavior. They explain the
  non-interactive Codex approval behavior but remain untrusted hints, not a
  security guarantee.
- OpenAI Codex #14242 now contains this investigation's deterministic custom
  Responses-provider namespace reproduction and workaround.

## Remaining external limitations

Provider authentication/anti-bot restrictions cannot be solved honestly inside Formal AI. A future Google capture can begin working without a Rust change when web-capture returns `status: ok` turns. Amazon availability must be checked by a human browser or an authorized Amazon API; the captured block explicitly directs automated users to Amazon's API support.

The four-client release test uses `.example.test` MCP fixture pages to make
protocol regressions deterministic. Native network runs remain in the evidence
bundle and are not conflated with current product verification. A separate
wall-clock SLA is not asserted because client/network latency is external, but
per-turn timestamps now make any future ten-second gap measurable.
