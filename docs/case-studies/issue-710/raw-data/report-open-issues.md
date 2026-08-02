# Open issues digest (31 open, as of 2026-07-14)

Structure: umbrella #651 + E37–E55 sub-issues (#656–#674), four recent (#687, #686, #682, #681), seven older (#557, #534, #531, #491, #483, #453, #447).

## Key requirements per issue

- **#687** (bug): "report the issue on GitHub" and similar meta-requests must work by generalization + auto-learning; talk about the conversation; web search/fetch to official sources; in agentic mode rely on harness tools (OpenCode etc.); every UI button/action/setting must be actionable via natural language in all environments.
- **#686**: persistent meta-language expressions in associative links networks per HF paper 2512.00590; count usages (reads) and changes (writes) via incoming/outgoing links; frequently used/changed data persists longer; everything a link (not graph/edges/vertices). PR #689 in flight.
- **#682** (bug): `"content": null` on assistant tool-call turns → 400, breaks qwen. Verified serde fix supplied. PR #685 in flight.
- **#681** (bug): file-creation requests emit `read` tool_call instead of `write` (1/50 write runs succeeded). Never route creation to read. PR #684 in flight (konard 07-14 comment "resolve conflicts and fully implement vision" unaddressed).
- **#651** (umbrella): roadmap must track requirement status (done/partial/not done) not issues; be ambitious; associative-only tech (no graphs/tables); self-coding via Agent CLI directed by Hive Mind; sub-issues with explicit GitHub blocking relationships; wide audience web/desktop.
- **#656 E37**: benchmark-gated promotion protocol — proposals passing ratchets materialize as .lino seed edits on branch via `formal-ai improve --promote`; "without it, learning cannot compound". PR #690 in flight.
- **#657 E38**: self-hosting metric — share of each release authored by Formal AI, ledger + release notes, monotonic ratchet, 0% honest start.
- **#658 E39**: absorb ~26.7k JS worker lines into Rust→WASM; JS ≤3k lines UI/glue; blocker for #665. PR #691 (groundwork) in flight.
- **#659 E40**: CI lint on hardcoded NL strings + allowlist burn-down. PR #692 in flight.
- **#660 E41**: bulk lexeme importer from Wikidata, ≥100 grounded meanings, en/ru/hi/zh, offline reproducible. PR #693 in flight (CI fixing).
- **#661 E42**: probability-weighted statement formalization + contradiction warnings with proposed resolutions (R384). PR #694 in flight.
- **#662 E43**: budget-driven random+evolutionary search (F4), compute_budget knob, deterministic, search: events. PR #695 in flight.
- **#663 E44**: retire SPECIALIZED_HANDLERS into handler-precedence.lino seed. PR #696 draft.
- **#664 E45**: terminology links-network-not-graph; /v1/network; lint. PR #697 draft.
- **#665 E46**: offline PWA + npm @link-assistant/formal-ai-engine (blocked by #658).
- **#666 E47**: publish VS Code extension to Marketplace + Open VSX.
- **#667 E48**: interactive 4-pane step-through debugger (chat/data/mermaid/Rust-JS), R383 (depends #666, #559 registry).
- **#668 E49**: shareable permissioned associative packages (F6).
- **#669 E50**: vendor-neutral opt-in cloud memory sync (F3), append-only conflict-free.
- **#670 E51**: WebVM/Pyodide browser execution experiment (F5), go/no-go.
- **#671 E52**: multi-CLI agentic e2e CI matrix (codex, opencode, gemini, qwen, claude, grok, aider + own Agent CLI) using PR-#631 recording proxy; regressions for #680/#681/#682/#650 defects.
- **#672 E53**: deferred issue-541 UI follow-ups F1–F5 (dark-theme snapshots, migration-replay UI, animation budget, reasoning-hierarchy editing, desktop IPC tests); reconcile with #557.
- **#673 E54**: workspace-wide self-AST census (planner sees whole workspace; needed for self-coding E35/E36 chain).
- **#674 E55**: compile arbitrary freely-phrased NL procedures to typed skills; named skill_gap on failure; vocabulary as seed data; same skill links from en/ru/hi/zh.
- **#557**: adaptive polished UI, embedded buttons in text field, multiple skins (default/glass/material) + transparency slider, best-rated UI kits, light/dark. PR #643 blocked on konard's "APPROVED TO FINALIZE" gate.
- **#534** (bug): 12G dev workspace; root cause (rust target? test cleanup?); repo download size; reduce compile size.
- **#531**: patterns inference via associative deduplication; port C# Data.Doublets.Sequences to Rust; rotation/translation-invariant sequence inference; everything is a link, analogy is a link; links meta-theory; seed ontology w/ recursive formal definitions proved via relative-meta-logic; apply dedup everywhere incl. self-improvement + converting event sequences to algorithms; learn meta algorithm from own code-change history. First session research/proposals only, konard decides. PR #642 (implementation demanded and addressed) in flight.
- **#491**: principle of least action — optimize reasoning path length; always split each task into 2 sub-tasks recursively (balanced binary tree); minimize total smallest sub-tasks.
- **#483**: opt-in small-model formalization fallback (choose among options, unit-test confirmed; off by default; hardware-filtered; download on demand; "LLMs never at steering wheel"). PR #644 in flight.
- **#453**: moonshot tasks — recursive 2-way splitting using best internet data, dedup ideas + trace to first historical source; moonshots: Atari Breakout 860-864 architecture; symbolic ChatGPT-like + benchmark; "write a strong AI"; weak intelligence (no own will) able to produce strong intelligence on demand.
- **#447** (bug): resizer/scroll confusion in web UI; VS-Code-style thin resizer. PR #646 in flight.

## Themes
(1) generality over special-casing; (2) self-improvement ladder gated by benchmarks; (3) agentic CLI control + multi-CLI CI matrix; (4) associative-only, behavior as seed data; (5) usage-weighted world-model memory; (6) multilingual en/ru/hi/zh by construction; (7) honesty/determinism/opt-in; (8) formal-first, LLM never in control; (9) budgeted/parallel search + least action; (10) wide delivery (PWA, npm, VS Code, desktop); (11) case-study + single-PR discipline; (12) blocking graph: #658→#665, #666→#667, #673→#656→#657, #671 guards #681/#682.

## Standard konard process clauses (apply to any new issue I create)
- Collect data to `./docs/case-studies/issue-{id}` (timeline, requirements list, root causes, solution plans, existing-library survey, online research).
- Add debug/verbose output if root cause undeterminable.
- File upstream issues with repros where applicable.
- Execute in a single PR until every requirement fully addressed.
- Detailed "what to do exactly and how to test/check it is done".
- Sub-issue of #651 where appropriate + explicit blocking relationships.
