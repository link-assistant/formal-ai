# Issue #989 — reported-dialog regression repair

Issue: <https://github.com/link-assistant/formal-ai/issues/989>

Pull request: <https://github.com/link-assistant/formal-ai/pull/998>

Issue #989 supplied a complete Formal AI / Agent CLI dialog and asked that
every error, warning, false positive, and false negative in it be corrected.
The issue comment further requires data-driven dialog hints, a deep case study,
and three separate context-file links in future reports.

## Preserved evidence

The original issue, its complete comment list, and the linked gist are retained
under `raw-data/`. The 42,563-byte `reported-session.log` is byte-identical to
the gist download (SHA-256
`79fd506ff1806eeb5197fc18b53def02e1f29eec1048175de1939c250e3e453a`).
It contains 1,438 lines and was read completely before implementation.

The initial state of every PR review surface is preserved separately under
`../pull-request-998/raw-data/`. No issue or PR discussion was omitted.

## Reproduction

The minimum failing turns from the report were converted into behavioral tests
in `tests/unit/issue_989.rs` and the browser-mirror test. Two independent red
runs are retained in `test-evidence/`:

- `red-conversation-preference.log` shows a dialog preference being classified
  as `web_search` instead of `conversation_preference`;
- `red-search-transport.log` shows a failed Exa request being rendered as
  “Research completed … but the tool returned no content.”

The whole-dialog regression replays all affected capability families in one
test so a future precedence change cannot fix each isolated prompt while
breaking their combined contract.

## Root causes

The reported failures were related by routing precedence and incomplete local
grammars, not by one defective response:

1. Local location phrases were absent from the shell-intent seed, so generic
   search captured them.
2. The document recipe treated the noun “Links Notation” as an action request;
   merely asking for its definition started file generation.
3. `Progress` retained a failed search/fetch result, but the search and research
   planners checked only whether the capability had completed. A transport
   failure therefore became a false successful summary.
4. Conversation preferences and unauthorized-change corrections had no local
   handler. Agentic planning ran before the symbolic dispatcher, and the
   dispatcher input still contained punctuation when role matching occurred.
5. Associative-memory inspection ran after agentic planning, allowing generic
   document or web routes to preempt a local read.
6. The skill compiler accepted only two backtick-delimited code spans. The
   unquoted teaching rule in the report was rejected even though its trigger
   and response verbs were explicit.
7. Behavior-rule detail parsing and seed surfaces used only the US spelling.
8. Agentic narration contained the subjective word “quick” after the user had
   explicitly rejected it.
9. The report command exposed one merged attachment. Passing multiple files to
   one `gh gist create` call still creates one gist, so it cannot satisfy a
   request for three distinct links.
10. Status-less tool results used a failure lexicon across the entire payload.
    A successful research page whose 404th evidence label contained `404` was
    therefore misclassified as a transport error during Agent CLI verification.

## Implementation

The fix keeps recognition data-driven across English, Russian, Hindi, Chinese,
and Spanish:

- new seed roles cover dialog preferences, corrections, associative-memory
  count/inventory/root queries, and correction turns;
- `pwd` gains the natural location cues from the report;
- conversation control and memory inspection run before generic agentic routes;
- the browser worker mirrors the native associative-memory projection and
  handler order;
- formalization requires an explicit formalization action;
- failed search and fetch observations stop with their real diagnostic;
- prose-only failure inference is bounded to the payload's leading diagnostic
  region, so error notices remain detectable without classifying later document
  contents as transport status;
- the skill compiler derives the unquoted teaching separators from seeded
  roles, including the longest `answer with` form;
- both behavior/behaviour spellings are recognized, and subjective narration
  was replaced in every supported-language response family;
- `formal-ai report body --separate-context-links` exports harness, server, and
  merged context separately, uploads each independently, sanitizes filenames,
  and renders link-only attachments without empty Markdown fences.

Existing context extractors, Links Notation conversion, memory-event projection,
localized response catalog, role lexicon, report renderer, and tool-failure
renderer were reused. No parallel memory store, report schema, or parser was
introduced.

## Verification

The final focused Rust run passes all 14 issue tests. The issue-specific browser
mirror regression passes alongside the 14 pre-existing mirror checks. The exact
logs are retained as `test-evidence/focused-green.log` and
`test-evidence/browser-green.log`. The requirements-to-test mapping is in
`requirements.md`.

Repository contracts also check:

- exact example answers for behavioral tests;
- Rust formatting and Clippy;
- generated role and browser-seed consistency;
- the browser worker line-budget ratchet;
- neighboring formalization, reporting, tool-failure, and memory tests;
- the full Rust and JavaScript suites.

## Same-task self-application

The reviewed task decomposition has five smallest leaves. A real external Agent
CLI, driven only through a locally built Formal AI server, authors the reviewed
decomposition leaf in session `ses_00fc002c9ffetId67J25NEeudn`. The repeatable runner is
`experiments/issue_989_self_authoring/run.sh`; the generated artifact is checked
byte-for-byte against its retained session copy. This is one of five leaves
(20%), satisfying the repository's self-application requirement without
misattributing the human-authored implementation.

## Timeline

- 2026-08-09/10: the reported dialog was captured against Formal AI v0.333.2.
- 2026-08-10: issue #989 was opened with the failing report.
- 2026-08-11: the issue comment supplied the full gist and reporting/case-study
  requirements; PR #998 was prepared and the tests-first repair began.
